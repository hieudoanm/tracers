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
                    // `let x = a.b` records the field path as the type
                    // expression; resolve it through declared field types.
                    let typed = if type_name.contains('.') {
                        self.resolve_type_expr(
                            index,
                            state,
                            resolver,
                            call.caller,
                            &type_name,
                            call.range.start.byte,
                        )
                    } else {
                        match resolver.resolve_type_name(state, call.caller, &type_name) {
                            crate::resolve::Resolution::Internal(symbol) if symbol.is_type() => {
                                Some(symbol)
                            }
                            crate::resolve::Resolution::External { qualified, .. } => {
                                return CallTarget::External(format!("{qualified}.{}", call.name));
                            }
                            _ => None,
                        }
                    };
                    if let Some(symbol) = typed {
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
                        if let ModuleRef::File(path) = module {
                            if let Some(target) = index.files.get(path) {
                                if let Some(symbol) =
                                    target.top_level.get(&call.name).and_then(|indexes| {
                                        indexes.first().and_then(|i| target.symbols.get(*i))
                                    })
                                {
                                    return CallTarget::Internal(symbol.node_id.clone());
                                }
                            }
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
    /// `pos.side_to_move`) to a type symbol by walking declared field types.
    /// Returns `None` when any hop lacks type info.
    fn resolve_type_expr(
        &self,
        index: &RepositoryIndex,
        state: &FileState,
        resolver: &Resolver,
        scope: Option<u32>,
        expr: &str,
        at_byte: u32,
    ) -> Option<SymbolRef> {
        use crate::resolve::Resolution;

        let mut segments = expr.split('.');
        let base = segments.next()?;
        let mut ty = if matches!(base, "self" | "this" | "Self") {
            index.parent_type_of(state, scope?)?
        } else {
            let type_name = resolver
                .local_type(state, scope, base, at_byte)
                .or_else(|| {
                    (base.chars().next().is_some_and(|c| c.is_uppercase()))
                        .then(|| base.to_string())
                })?;
            match resolver.resolve_type_name(state, scope, &type_name) {
                Resolution::Internal(symbol) if symbol.is_type() => symbol,
                _ => return None,
            }
        };
        for segment in segments {
            let field_type = resolver.field_type(&ty, segment)?;
            match resolver.resolve_type_name(state, scope, &field_type) {
                Resolution::Internal(symbol) if symbol.is_type() => ty = symbol,
                _ => return None,
            }
        }
        Some(ty)
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

        let ty = self.resolve_type_expr(
            index,
            state,
            resolver,
            call.caller,
            receiver,
            call.range.start.byte,
        )?;
        if let Resolution::Internal(member) =
            resolver.resolve_member(&ty, &call.name, Want::Callable)
        {
            return Some(CallTarget::Internal(member.node_id));
        }
        self.find_type_member(index, &ty, &call.name, false)
            .map(|member| CallTarget::Internal(member.node_id))
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
        let matches: Vec<&SymbolRef> = candidates
            .iter()
            .filter(|s| !callable_only || s.is_callable())
            .collect();
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
