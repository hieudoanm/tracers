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
        let name = name.into();
        if name.is_empty() || super::super::builtins::is_builtin_type(&name) {
            return;
        }
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
        self.heritage.push(HeritageData {
            from,
            name: name.into(),
            relation,
            range: super::super::tree_sitter::range(node),
        });
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
