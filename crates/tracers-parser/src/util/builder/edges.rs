use tree_sitter::Node as TsNode;

use crate::syntax::{CallData, CallKind, HeritageData, HeritageRelation, ImportData, TypeRefData};

use super::SyntaxBuilder;

impl<'a> SyntaxBuilder<'a> {
    pub(crate) fn add_import(&mut self, import: ImportData) {
        self.imports.push(import);
    }

    pub(crate) fn add_call(
        &mut self,
        receiver: Option<String>,
        name: impl Into<String>,
        full: impl Into<String>,
        kind: CallKind,
        node: TsNode<'_>,
    ) {
        let Some(name) = clean_call_name(&name.into()) else {
            return;
        };
        self.calls.push(CallData {
            caller: self.caller(),
            receiver,
            name,
            full: full.into(),
            kind,
            range: super::super::tree_sitter::range(node),
        });
    }

    pub(crate) fn add_type_ref(&mut self, name: impl Into<String>, node: TsNode<'_>) {
        let Some(name) = clean_symbol_name(&name.into()) else {
            return;
        };
        // Single uppercase letters in generic languages are type parameters
        // (`[T any]`, `<T>`, `T extends ...`) that bind lexically rather
        // than referencing a declared type.
        if name.len() == 1
            && name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && matches!(
                self.language,
                tracers_core::Language::Rust
                    | tracers_core::Language::Go
                    | tracers_core::Language::Kotlin
                    | tracers_core::Language::Swift
                    | tracers_core::Language::Java
                    | tracers_core::Language::CSharp
                    | tracers_core::Language::TypeScript
                    | tracers_core::Language::Tsx
                    | tracers_core::Language::Cpp
                    | tracers_core::Language::Dart
                    | tracers_core::Language::Scala
            )
        {
            return;
        }
        // Builtin/primitive names are kept: analysis maps known stdlib types
        // to `resolved_external` edges and drops the rest.
        self.type_refs.push(TypeRefData {
            from: self.caller(),
            name,
            range: super::super::tree_sitter::range(node),
        });
    }

    pub(crate) fn add_heritage(
        &mut self,
        from: u32,
        name: impl Into<String>,
        relation: HeritageRelation,
        node: TsNode<'_>,
    ) {
        let Some(name) = clean_symbol_name(&name.into()) else {
            return;
        };
        self.heritage.push(HeritageData {
            from,
            name,
            relation,
            range: super::super::tree_sitter::range(node),
        });
    }
}

/// Return `name` when it is a plausible type/symbol reference name.
///
/// Adapter fallbacks occasionally slice raw expression text, leaking
/// char literals (`'a`), multi-line chains, or generic argument lists
/// into type references and heritage names. Qualified names keep their
/// `.`/`::` separators; trailing `?`/`!` cover Swift optionals and
/// Rust never-type/never-return markers.
fn clean_symbol_name(name: &str) -> Option<String> {
    let name = name.trim();
    if name.is_empty() || name.len() > 200 {
        return None;
    }
    let core = name.trim_end_matches(['?', '!']);
    if !core.is_empty()
        && core
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | '$' | '.' | ':'))
    {
        Some(name.to_string())
    } else {
        None
    }
}

/// Return `name` when it is a plausible terminal callee identifier.
///
/// Parser fallbacks occasionally slice raw expression text, leaking
/// multi-line call chains, char literals, or generic arguments into the
/// callee name (e.g. `"<String>()\n .split('-')"`). Such names poison
/// `by_name` lookups and `unresolved` placeholder nodes, so calls with
/// non-identifier names are dropped instead of recorded.
fn clean_call_name(name: &str) -> Option<String> {
    let trimmed = name.trim();
    let name = trimmed.strip_prefix("r#").unwrap_or(trimmed);
    // Ruby predicate/bang/setter methods legitimately end in `?`, `!`, `=`.
    let core = name.trim_end_matches(['?', '!', '=']);
    if !core.is_empty()
        && core
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
    {
        Some(name.to_string())
    } else {
        None
    }
}
