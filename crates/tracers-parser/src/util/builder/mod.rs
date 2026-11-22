pub mod edges;
pub mod finish;
pub mod locals;
pub mod symbols;

use std::collections::HashMap;

use tree_sitter::Node as TsNode;

use tracers_core::{Language, NodeKind};

use crate::syntax::{CallData, HeritageData, ImportData, SymbolData, TypeRefData};

pub struct SyntaxBuilder<'a> {
    pub(crate) source: &'a str,
    pub(crate) language: Language,
    pub(crate) path: &'a str,
    pub(crate) module_name: Option<String>,
    pub(crate) symbols: Vec<SymbolData>,
    pub(crate) imports: Vec<ImportData>,
    pub(crate) calls: Vec<CallData>,
    pub(crate) type_refs: Vec<TypeRefData>,
    pub(crate) heritage: Vec<HeritageData>,
    pub(crate) locals: Vec<crate::syntax::LocalBinding>,
    stack: Vec<u32>,
    qualifier_prefix: Option<String>,
    ordinals: HashMap<(NodeKind, String), u32>,
    next_local: u32,
    generic_names: Vec<String>,
    generic_marks: Vec<usize>,
}

impl<'a> SyntaxBuilder<'a> {
    pub(crate) fn new(source: &'a str, language: Language, path: &'a str) -> Self {
        Self {
            source,
            language,
            path,
            module_name: None,
            symbols: Vec::new(),
            imports: Vec::new(),
            calls: Vec::new(),
            type_refs: Vec::new(),
            heritage: Vec::new(),
            locals: Vec::new(),
            stack: Vec::new(),
            qualifier_prefix: None,
            ordinals: HashMap::new(),
            next_local: 0,
            generic_names: Vec::new(),
            generic_marks: Vec::new(),
        }
    }

    pub(crate) fn text(&self, node: TsNode<'_>) -> &'a str {
        super::tree_sitter::text(self.source, node)
    }

    pub(crate) fn caller(&self) -> Option<u32> {
        self.stack.last().copied()
    }

    pub(crate) fn current_symbol(&self) -> Option<&SymbolData> {
        self.stack
            .last()
            .and_then(|id| self.symbols.iter().find(|s| s.local_id == *id))
    }

    pub(crate) fn parent_local(&self) -> Option<u32> {
        self.stack.last().copied()
    }

    pub(crate) fn in_type_scope(&self) -> bool {
        self.qualifier_prefix.is_some()
            || self.current_symbol().is_some_and(|symbol| {
                matches!(
                    symbol.kind,
                    NodeKind::Class
                        | NodeKind::Struct
                        | NodeKind::Enum
                        | NodeKind::Interface
                        | NodeKind::Trait
                )
            })
    }

    pub(crate) fn set_qualifier(&mut self, qualifier: Option<String>) -> Option<String> {
        std::mem::replace(&mut self.qualifier_prefix, qualifier)
    }

    #[allow(dead_code)]
    pub(crate) fn find_symbol_by_id(&self, local_id: u32) -> Option<&SymbolData> {
        self.symbols
            .iter()
            .find(|symbol| symbol.local_id == local_id)
    }

    pub(crate) fn push(&mut self, local_id: u32) {
        self.stack.push(local_id);
    }

    pub(crate) fn pop(&mut self) {
        self.stack.pop();
    }

    /// Record the type/const generic parameter names declared by `node`'s
    /// `type_parameters` so uses of them (`[T; N]`, `Vec<T>`) are not
    /// mistaken for references to declared types. Pair with `pop_generics`.
    pub(crate) fn push_generics(&mut self, node: TsNode<'_>) {
        self.generic_marks.push(self.generic_names.len());
        let Some(parameters) = super::child_field(node, "type_parameters") else {
            return;
        };
        for parameter in super::named_children(parameters) {
            if !matches!(parameter.kind(), "type_parameter" | "const_parameter") {
                continue;
            }
            if let Some(name) = super::child_field(parameter, "name") {
                self.generic_names.push(self.text(name).to_string());
            }
        }
    }

    pub(crate) fn pop_generics(&mut self) {
        if let Some(mark) = self.generic_marks.pop() {
            self.generic_names.truncate(mark);
        }
    }

    pub(crate) fn is_generic_param(&self, name: &str) -> bool {
        self.generic_names.iter().any(|param| param == name)
    }
}
