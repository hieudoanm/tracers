//! Helper methods for GraphBuilder.

use tracers_core::{Graph, Language, Node, NodeKind};
use tracers_parser::{CallData, CallKind};

use crate::index::{FileState, ModuleRef, RepositoryIndex, SymbolRef};
use crate::resolve::resolve_path_helpers::last_path_segment;
use crate::resolve::resolve_resolver::{Resolver, clean_name};

use super::graph_types::{CallTarget, GraphBuilder};

/// Resolve the target of a call expression.
impl GraphBuilder {
    #[allow(clippy::collapsible_match)]
    pub(crate) fn resolve_call_target(
        &self,
        index: &RepositoryIndex,
        state: &FileState,
        resolver: &Resolver,
        call: &CallData,
    ) -> CallTarget {
        // If there is a receiver, resolve the receiver type and look up the
        // member method on it.
        if let Some(receiver) = &call.receiver {
            let cleaned = clean_name(receiver);
            // `self.foo()` / `this.foo()`: the member lives on the enclosing
            // type, so look up the call name — not the receiver text.
            if matches!(cleaned.as_str(), "self" | "this" | "Self") {
                if let Some(scope) = call.caller {
                    if let Some(parent) = index.parent_type_of(state, scope) {
                        let member = resolver.resolve_member(
                            &parent,
                            &call.name,
                            crate::resolve::resolve_types::Want::Callable,
                        );
                        if let crate::resolve::Resolution::Internal(symbol) = member {
                            return CallTarget::Internal(symbol.node_id);
                        }
                    }
                }
            }
            // Receiver is a local with a recorded type
            // (`position.get_piece()` where `position: &Position`).
            if !cleaned.contains("::") && !cleaned.contains('.') {
                if let Some(type_name) =
                    resolver.local_type(state, call.caller, &cleaned, call.range.start.byte)
                {
                    // `x := f()` locals carry a `@call:` sentinel resolved
                    // through the callee's declared result types — in the
                    // callee file's context, since result qualifiers refer
                    // to the callee's imports (`db.NewSession` → `*gocql.Session`).
                    let resolution = if type_name.starts_with("@call:") {
                        self.expand_call_type(
                            index,
                            state,
                            resolver,
                            call.caller,
                            call.range.start.byte,
                            &type_name,
                        )
                    } else {
                        // Qualified type names (`models.Foo`, `gocql.Session`)
                        // resolve as types first; a field path (`pos.side`)
                        // falls back to walking declared field types.
                        match resolver.resolve_type_name(state, call.caller, &type_name) {
                            crate::resolve::Resolution::Internal(symbol) if symbol.is_type() => {
                                Some(crate::resolve::Resolution::Internal(symbol))
                            }
                            crate::resolve::Resolution::External { qualified, .. } => {
                                return CallTarget::External(format!("{qualified}.{}", call.name));
                            }
                            _ if type_name.contains('.') => self.resolve_type_expr(
                                index,
                                state,
                                resolver,
                                call.caller,
                                &type_name,
                                call.range.start.byte,
                            ),
                            _ => None,
                        }
                    };
                    match resolution {
                        Some(crate::resolve::Resolution::Internal(symbol)) => {
                            // `resolve_member` walks supertypes, so trait
                            // default methods (`visitor.visit_source_file`)
                            // resolve through `impl Trait for Type`.
                            if let crate::resolve::Resolution::Internal(member) = resolver
                                .resolve_member(
                                    &symbol,
                                    &call.name,
                                    crate::resolve::resolve_types::Want::Callable,
                                )
                            {
                                return CallTarget::Internal(member.node_id);
                            }
                            if let Some(member) =
                                self.find_type_member(index, &symbol, &call.name, false)
                            {
                                return CallTarget::Internal(member.node_id);
                            }
                        }
                        Some(crate::resolve::Resolution::External { qualified, .. }) => {
                            return CallTarget::External(format!("{qualified}.{}", call.name));
                        }
                        _ => {}
                    }
                }
            }
            // Dotted receiver (`position.side_to_move.opponent()`,
            // `self.engine.position.get_piece()`): walk declared field
            // types from the base local/self type, then resolve the member.
            if cleaned.contains('.') && !cleaned.contains("::") {
                if let Some(target) =
                    self.resolve_dotted_receiver(index, state, resolver, call, &cleaned)
                {
                    return target;
                }
            }
            // Receiver resolved as a type in bindings.
            if let Some(binding) = state.bindings.get(&cleaned) {
                match binding {
                    crate::index::Binding::Symbol(symbol) if symbol.is_type() => {
                        if let Some(member) =
                            self.find_method_on_type(index, state, symbol, &call.name)
                        {
                            return CallTarget::Internal(member.node_id);
                        }
                        // Enum variants and other non-callable members
                        // (`AppError::Internal`, `Position::StartPos`).
                        if let Some(member) =
                            self.find_type_member(index, symbol, &call.name, false)
                        {
                            return CallTarget::Internal(member.node_id);
                        }
                        // The receiver is a known internal type; an unknown
                        // member is honestly unresolved, not external.
                        return CallTarget::Unresolved;
                    }
                    crate::index::Binding::Module(module) => {
                        // `pkg.Func()` / `mod.func()`: member lookup inside
                        // the bound module — file modules check exports and
                        // top-level symbols, package modules search every
                        // file in the package, external modules produce a
                        // qualified external edge.
                        match resolver.resolve_module_member(state, module, &call.name) {
                            crate::resolve::Resolution::Internal(symbol) => {
                                return CallTarget::Internal(symbol.node_id);
                            }
                            crate::resolve::Resolution::External { qualified, .. } => {
                                return CallTarget::External(qualified);
                            }
                            crate::resolve::Resolution::ExternalNode { node_id, .. } => {
                                return CallTarget::Internal(node_id);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            // Path-qualified receiver (`crate::util::child_field`,
            // `module::func`): resolve the qualifier as a module path and
            // look the member up inside it.
            if cleaned.contains("::") {
                // `crate::Type::Variant` / `module::Type::assoc`: the
                // qualifier may be a type path rather than a module path.
                if let crate::resolve::Resolution::Internal(symbol) =
                    resolver.resolve_type_name(state, call.caller, &cleaned)
                {
                    if symbol.is_type() {
                        if let crate::resolve::Resolution::Internal(member) = resolver
                            .resolve_member(
                                &symbol,
                                &call.name,
                                crate::resolve::resolve_types::Want::Callable,
                            )
                        {
                            return CallTarget::Internal(member.node_id);
                        }
                        if let Some(member) =
                            self.find_type_member(index, &symbol, &call.name, false)
                        {
                            return CallTarget::Internal(member.node_id);
                        }
                        return CallTarget::Unresolved;
                    }
                }
                // Rust `crate::`/`self::`/`super::` paths and module
                // re-exports (`crate::ica::fastica`).
                if let Some(resolution) = resolver.resolve_rust_path(
                    state,
                    &format!("{cleaned}::{}", call.name),
                    crate::resolve::resolve_types::Want::Callable,
                ) {
                    match resolution {
                        crate::resolve::Resolution::Internal(symbol) => {
                            return CallTarget::Internal(symbol.node_id);
                        }
                        crate::resolve::Resolution::External { qualified, .. } => {
                            return CallTarget::External(qualified);
                        }
                        crate::resolve::Resolution::ExternalNode { node_id, .. } => {
                            return CallTarget::Internal(node_id);
                        }
                        _ => {}
                    }
                }
                let ctx = crate::resolve::resolve_import_pass::ImportContext {
                    index,
                    state,
                    path: state.path.clone(),
                    language: state.language,
                    diagnostics: Vec::new(),
                    edges: Vec::new(),
                    package_pairs: Default::default(),
                };
                if let Some(module) = ctx.resolve_module_path(&cleaned) {
                    let allow_private = ctx.allow_private(&module);
                    match ctx.resolve_module_name(&module, &call.name, allow_private, None) {
                        crate::resolve::Resolution::Internal(symbol) => {
                            return CallTarget::Internal(symbol.node_id);
                        }
                        crate::resolve::Resolution::External { qualified, .. } => {
                            return CallTarget::External(qualified);
                        }
                        crate::resolve::Resolution::ExternalNode { node_id, .. } => {
                            return CallTarget::Internal(node_id);
                        }
                        _ => {}
                    }
                }
            }
            // `t.Fatalf` in Go test files → `testing.T.Fatalf`.
            if let Some(qualified) =
                crate::resolve::builtin_calls::go_testing_target(&state.path, &cleaned, &call.name)
            {
                return CallTarget::External(qualified);
            }
            // Receiver didn't resolve internally: it may be a stdlib or
            // manifest-declared package qualifier (`fmt.Println`,
            // `serde_json::json!`, `os.path.join`, `console.log`).
            if let Some(qualified) = crate::resolve::builtin_calls::receiver_external_target(
                state.language,
                &cleaned,
                &call.name,
                &index.external_deps,
            ) {
                return CallTarget::External(qualified);
            }
            return match call.kind {
                CallKind::Constructor => CallTarget::External(format!("{receiver}.{}", call.name)),
                _ => crate::resolve::builtin_calls::builtin_qualified(state.language, &call.name)
                    .map(CallTarget::External)
                    .unwrap_or(CallTarget::Unresolved),
            };
        }

        // Plain function call: resolve the name through bindings and scope.
        let scope = call.caller;
        let resolution = resolver.resolve_callable_name(state, scope, &call.name);
        match resolution {
            crate::resolve::Resolution::Internal(symbol) => {
                return CallTarget::Internal(symbol.node_id);
            }
            crate::resolve::Resolution::ExternalNode { node_id, .. } => {
                return CallTarget::Internal(node_id);
            }
            crate::resolve::Resolution::Module(module) => {
                return match module {
                    ModuleRef::File(path) => {
                        if let Some(target) = index.files.get(path.as_str()) {
                            if let Some(symbol) =
                                target.top_level.get(&call.name).and_then(|indexes| {
                                    indexes.first().and_then(|i| target.symbols.get(*i))
                                })
                            {
                                CallTarget::Internal(symbol.node_id.clone())
                            } else {
                                CallTarget::Unresolved
                            }
                        } else {
                            CallTarget::Unresolved
                        }
                    }
                    ModuleRef::Package(_) => CallTarget::Unresolved,
                    ModuleRef::External(name) => {
                        CallTarget::External(format!("{name}.{}", call.name))
                    }
                };
            }
            crate::resolve::Resolution::Ambiguous { candidates, .. } => {
                return CallTarget::Ambiguous(candidates);
            }
            _ => {}
        }

        // Same-package/group fallback for callables.
        if let Some(group) = &state.group {
            let candidates = index.symbols_in_group(group, &call.name, false);
            let callables: Vec<_> = candidates.into_iter().filter(|s| s.is_callable()).collect();
            match callables.len() {
                1 => return CallTarget::Internal(callables[0].node_id.clone()),
                _ if !callables.is_empty() => {
                    return CallTarget::Ambiguous(
                        callables.into_iter().map(|s| s.node_id).collect(),
                    );
                }
                _ => {}
            }
        }

        // Last resort: the callee may be a non-callable-kind binding —
        // destructured hook setters (`const [x, setX] = useState()`),
        // function-valued variables, enum constructors. An explicit binding
        // beats builtin/external guesses.
        if let crate::resolve::Resolution::Internal(symbol) = resolver.resolve_name(
            state,
            scope,
            &call.name,
            crate::resolve::resolve_types::Want::Any,
        ) {
            return CallTarget::Internal(symbol.node_id);
        }

        match call.kind {
            CallKind::Constructor => CallTarget::External(
                crate::resolve::builtin_calls::builtin_qualified(state.language, &call.name)
                    .unwrap_or_else(|| call.name.clone()),
            ),
            _ => crate::resolve::builtin_calls::builtin_qualified(state.language, &call.name)
                .map(CallTarget::External)
                .unwrap_or(CallTarget::Unresolved),
        }
    }

    /// Resolve a possibly dotted type expression (`position`, `self.engine`,
    /// `pos.side_to_move`) to a type resolution by walking declared field
    /// types. Returns `None` when any hop lacks type info; an `External`
    /// resolution is only produced by the final hop.
    fn resolve_type_expr(
        &self,
        index: &RepositoryIndex,
        state: &FileState,
        resolver: &Resolver,
        scope: Option<u32>,
        expr: &str,
        at_byte: u32,
    ) -> Option<crate::resolve::Resolution> {
        use crate::resolve::Resolution;

        let mut segments = expr.split('.').peekable();
        let base = segments.next()?;
        let mut resolution = if matches!(base, "self" | "this" | "Self") {
            Resolution::Internal(index.parent_type_of(state, scope?)?)
        } else {
            let type_name = resolver
                .local_type(state, scope, base, at_byte)
                .or_else(|| {
                    (base.chars().next().is_some_and(|c| c.is_uppercase()))
                        .then(|| base.to_string())
                })?;
            if type_name.starts_with("@call:") {
                self.expand_call_type(index, state, resolver, scope, at_byte, &type_name)?
            } else {
                resolver.resolve_type_name(state, scope, &type_name)
            }
        };
        if let Resolution::Internal(symbol) = &resolution {
            if !symbol.is_type() {
                return None;
            }
        } else if !matches!(resolution, Resolution::External { .. }) {
            return None;
        }
        while let Some(segment) = segments.next() {
            let is_last = segments.peek().is_none();
            let Resolution::Internal(symbol) = &resolution else {
                // External types have no visible fields; only the final
                // hop may land on one.
                return if is_last { Some(resolution) } else { None };
            };
            let field_type = resolver.field_type(symbol, segment)?;
            resolution = resolver.resolve_type_name(state, scope, &field_type);
            match &resolution {
                Resolution::Internal(symbol) if symbol.is_type() => {}
                Resolution::External { .. } if is_last => return Some(resolution),
                _ => return None,
            }
        }
        Some(resolution)
    }

    /// Resolve `a.b.c.name()` where `a` is a local/self/type and `b`, `c`
    /// are field accesses. Returns `None` when any hop lacks type info.
    fn resolve_dotted_receiver(
        &self,
        index: &RepositoryIndex,
        state: &FileState,
        resolver: &Resolver,
        call: &CallData,
        receiver: &str,
    ) -> Option<CallTarget> {
        use crate::resolve::Resolution;
        use crate::resolve::resolve_types::Want;

        match self.resolve_type_expr(
            index,
            state,
            resolver,
            call.caller,
            receiver,
            call.range.start.byte,
        )? {
            Resolution::Internal(ty) => {
                if let Resolution::Internal(member) =
                    resolver.resolve_member(&ty, &call.name, Want::Callable)
                {
                    return Some(CallTarget::Internal(member.node_id));
                }
                self.find_type_member(index, &ty, &call.name, false)
                    .map(|member| CallTarget::Internal(member.node_id))
            }
            Resolution::External { qualified, .. } => {
                Some(CallTarget::External(format!("{qualified}.{}", call.name)))
            }
            _ => None,
        }
    }

    /// Expand a `@call:{callee}:{index}` local-type sentinel (emitted for
    /// `x := f()` / `a, b := f()` initializers) into the callee's declared
    /// result type at `index`, resolved in the *callee file's* context —
    /// result qualifiers like `*gocql.Session` refer to the callee's
    /// imports, not the caller's. Returns `None` for non-sentinel names,
    /// unresolvable callees, or result types that aren't types.
    fn expand_call_type(
        &self,
        index: &RepositoryIndex,
        state: &FileState,
        resolver: &Resolver,
        scope: Option<u32>,
        at_byte: u32,
        type_name: &str,
    ) -> Option<crate::resolve::Resolution> {
        use crate::resolve::Resolution;

        let rest = type_name.strip_prefix("@call:")?;
        let (callee, index_) = rest.rsplit_once(':')?;
        let result_index: usize = index_.parse().ok()?;
        let symbol = self.resolve_callee_symbol(index, state, resolver, scope, at_byte, callee)?;
        let callee_state = index.files.get(&symbol.path)?;
        let signature = callee_state
            .parsed
            .symbols
            .iter()
            .find(|data| data.local_id == symbol.local_id)
            .and_then(|data| data.signature.clone())?;
        let result_type = go_signature_results(&signature).get(result_index)?.clone();
        match resolver.resolve_type_name(callee_state, Some(symbol.local_id), &result_type) {
            Resolution::Internal(symbol) if symbol.is_type() => Some(Resolution::Internal(symbol)),
            resolution @ Resolution::External { .. } => Some(resolution),
            _ => None,
        }
    }

    /// Resolve a `@call:` sentinel callee to its symbol. Handles plain
    /// names (`newConsumer`), package qualifiers (`db.NewSession`), and
    /// receiver calls (`t.M`, `self.M`). External callees return `None` —
    /// their result types are unknowable without sources.
    fn resolve_callee_symbol(
        &self,
        index: &RepositoryIndex,
        state: &FileState,
        resolver: &Resolver,
        scope: Option<u32>,
        at_byte: u32,
        callee: &str,
    ) -> Option<crate::index::SymbolRef> {
        use crate::index::Binding;
        use crate::resolve::Resolution;
        use crate::resolve::resolve_types::Want;

        if !callee.contains('.') && !callee.contains("::") {
            return match resolver.resolve_callable_name(state, scope, callee) {
                Resolution::Internal(symbol) => Some(symbol),
                _ => None,
            };
        }
        let (receiver, name) = callee.rsplit_once('.')?;
        if receiver.contains('.') || receiver.contains("::") {
            return None;
        }
        // `pkg.Func`: qualifier binds to a module/package.
        if let Some(Binding::Module(module)) = state.bindings.get(receiver) {
            return match resolver.resolve_module_member(state, module, name) {
                Resolution::Internal(symbol) => Some(symbol),
                _ => None,
            };
        }
        // `recv.Method` / `self.Method`: receiver type, then member lookup.
        let ty = self.resolve_type_expr(index, state, resolver, scope, receiver, at_byte)?;
        let Resolution::Internal(ty) = ty else {
            return None;
        };
        if let Resolution::Internal(member) = resolver.resolve_member(&ty, name, Want::Callable) {
            return Some(member);
        }
        self.find_type_member(index, &ty, name, true)
    }

    fn find_method_on_type(
        &self,
        index: &RepositoryIndex,
        _state: &FileState,
        type_symbol: &SymbolRef,
        method_name: &str,
    ) -> Option<SymbolRef> {
        self.find_type_member(index, type_symbol, method_name, true)
    }

    /// Look up `Type.name` in `by_qualified`. With `callable_only` set,
    /// only function-like members match; otherwise any single member kind
    /// (enum variants are `Constant`) resolves.
    fn find_type_member(
        &self,
        index: &RepositoryIndex,
        type_symbol: &SymbolRef,
        member_name: &str,
        callable_only: bool,
    ) -> Option<SymbolRef> {
        let key = format!("{}.{}", type_symbol.qualified, member_name);
        let candidates = index.by_qualified.get(&key)?;
        let mut matches: Vec<&SymbolRef> = candidates
            .iter()
            .filter(|s| !callable_only || s.is_callable())
            .collect();
        // Same-package preference: qualified names aren't package-scoped,
        // so `T.M` can collide across directories (e.g. a Go struct method
        // and an interface method on a same-named type).
        if matches.len() > 1 {
            let type_dir = type_symbol.path.rsplit_once('/').map(|(dir, _)| dir);
            let same_dir: Vec<&SymbolRef> = matches
                .iter()
                .filter(|symbol| {
                    symbol
                        .path
                        .rsplit_once('/')
                        .map(|(dir, _)| Some(dir) == type_dir)
                        .unwrap_or(false)
                })
                .copied()
                .collect();
            if !same_dir.is_empty() {
                matches = same_dir;
            }
        }
        if matches.len() == 1 {
            return Some(matches[0].clone());
        }
        None
    }

    pub(crate) fn parent_node_id(&self, state: &FileState, symbol: &SymbolRef) -> Option<String> {
        let Some(parent_local) = symbol.parent else {
            return Some(state.module_id.clone());
        };
        let parent_symbol = state.symbol(parent_local)?;
        Some(parent_symbol.node_id.clone())
    }

    /// Resolve a module reference to the canonical node id used as an edge
    /// target.
    pub(crate) fn module_target_node(
        &self,
        index: &RepositoryIndex,
        module_id_str: &str,
    ) -> String {
        if module_id_str.starts_with("external:") {
            return module_id_str.to_string();
        }
        // Resolve a module id back to a file node when possible.
        if let Some(path) = index.module_files.get(module_id_str) {
            return tracers_core::file_id(path);
        }
        // Check package id.
        if index.packages.contains_key(module_id_str) {
            return module_id_str.to_string();
        }
        module_id_str.to_string()
    }

    pub(crate) fn add_external_node(&self, graph: &mut Graph, language: Language, qualified: &str) {
        let node_id = tracers_core::external_id(language, qualified);
        let name = last_path_segment(qualified);
        let node =
            Node::new(node_id, NodeKind::ExternalSymbol, name.to_string()).with_language(language);
        let _ = graph.add_node(node);
    }

    /// Ensure a placeholder `unresolved:<language>:<name>` node exists so
    /// unresolved edges always have a valid target. The language qualifier
    /// keeps same-named unresolved symbols in different languages distinct.
    pub(crate) fn add_unresolved_node(&self, graph: &mut Graph, language: Language, name: &str) {
        let node_id = unresolved_id(language, name);
        let node = Node::new(node_id, NodeKind::Unknown, name.to_string()).with_language(language);
        let _ = graph.add_node(node);
    }
}

/// Canonical placeholder id for an unresolved symbol reference.
pub(crate) fn unresolved_id(language: Language, name: &str) -> String {
    format!("unresolved:{}:{name}", language.as_str())
}

/// Extract declared result types from a Go function signature's first
/// line, e.g. `func f(a int) (*T, error) {` → `["T", "error"]`.
/// Returns an empty vec for signatures without results or ones too
/// malformed to parse (multi-line headers truncate at `{`).
fn go_signature_results(signature: &str) -> Vec<String> {
    let s = signature.trim();
    let bytes = s.as_bytes();
    let mut i = 0;
    let skip_ws = |mut i: usize| {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        i
    };
    // `func` keyword.
    if !s[i..].starts_with("func") {
        return Vec::new();
    }
    i = skip_ws(i + 4);
    // Optional receiver `(r *T)`.
    if bytes.get(i) == Some(&b'(') {
        let Some(end) = matching_delim(bytes, i, b'(', b')') else {
            return Vec::new();
        };
        i = skip_ws(end + 1);
    }
    // Function name.
    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    i = skip_ws(i);
    // Optional type parameters `[T any]`.
    if bytes.get(i) == Some(&b'[') {
        let Some(end) = matching_delim(bytes, i, b'[', b']') else {
            return Vec::new();
        };
        i = skip_ws(end + 1);
    }
    // Parameter list.
    if bytes.get(i) != Some(&b'(') {
        return Vec::new();
    }
    let Some(end) = matching_delim(bytes, i, b'(', b')') else {
        return Vec::new();
    };
    i = skip_ws(end + 1);
    // Result section runs to the body `{` (or end of the truncated line).
    let rest = &s[i..];
    let result_end = rest.find('{').unwrap_or(rest.len());
    let result = rest[..result_end].trim();
    if result.is_empty() {
        return Vec::new();
    }
    let inner = if result.starts_with('(') && result.ends_with(')') && result.len() > 2 {
        &result[1..result.len() - 1]
    } else {
        result
    };
    split_top_level_commas(inner)
        .into_iter()
        .map(|part| {
            part.trim()
                .trim_start_matches(['*', '&', '[', ']'])
                .trim()
                .to_string()
        })
        .filter(|part| !part.is_empty())
        .collect()
}

/// Index just past the delimiter matching the one at `start`, or `None`.
fn matching_delim(bytes: &[u8], start: usize, open: u8, close: u8) -> Option<usize> {
    let mut depth = 0usize;
    for (offset, &byte) in bytes.iter().enumerate().skip(start) {
        if byte == open {
            depth += 1;
        } else if byte == close {
            depth -= 1;
            if depth == 0 {
                return Some(offset);
            }
        }
    }
    None
}

/// Split on commas that are not nested inside `()`, `[]`, `{}`, or `<>`
/// (generics like `map[string]T` / `Foo[A, B]`).
fn split_top_level_commas(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    for c in input.chars() {
        match c {
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' | '>' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(std::mem::take(&mut current));
                continue;
            }
            _ => {}
        }
        current.push(c);
    }
    parts.push(current);
    parts
}

/// Return the source range for a symbol.
pub(crate) fn symbol_range(
    state: &FileState,
    symbol: &SymbolRef,
) -> Option<tracers_core::SourceRange> {
    state
        .parsed
        .symbols
        .iter()
        .find(|s| s.local_id == symbol.local_id)
        .map(|s| s.range)
}
