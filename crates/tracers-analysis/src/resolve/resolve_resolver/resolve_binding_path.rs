use super::super::resolve_import_pass::ImportContext;
use super::super::resolve_types::{Resolution, Want};
use crate::index::{Binding, FileState};

use super::Resolver;

impl<'a> Resolver<'a> {
    pub fn resolve_binding_path(
        &self,
        state: &FileState,
        raw: &str,
        want: Want,
    ) -> Option<Resolution> {
        let normalized = raw.replace("::", ".");
        let segments: Vec<&str> = normalized
            .split('.')
            .filter(|segment| !segment.is_empty())
            .collect();
        if segments.len() < 2 {
            return None;
        }
        for length in (1..=segments.len()).rev() {
            let key = segments[..length].join(".");
            let Some(binding) = state.bindings.get(&key) else {
                continue;
            };
            let mut resolution = match binding {
                Binding::Symbol(symbol) => Resolution::Internal(symbol.clone()),
                Binding::Module(module) => Resolution::Module(module.clone()),
                Binding::ExternalSymbol { node_id, name } => Resolution::ExternalNode {
                    node_id: node_id.clone(),
                    name: name.clone(),
                },
                Binding::Unresolved { name } => Resolution::Unresolved { name: name.clone() },
                Binding::Ambiguous { name, candidates } => Resolution::Ambiguous {
                    name: name.clone(),
                    candidates: candidates.clone(),
                },
            };
            for segment in &segments[length..] {
                resolution = self.navigate(state, resolution, segment, want)?;
            }
            return Some(resolution);
        }
        None
    }

    /// Resolve Rust `::` paths that are not import bindings:
    /// `crate::model::ApiRequest`, `self::foo::Bar`, `super::Baz`, or a
    /// plain `module::Type` path. Tries the longest module prefix first,
    /// then walks re-exported modules (`pub use other_crate::some_module`)
    /// recorded as `Binding::Module` in intermediate files.
    pub(crate) fn resolve_rust_path(
        &self,
        state: &FileState,
        name: &str,
        want: Want,
    ) -> Option<Resolution> {
        if state.language != tracers_core::Language::Rust || !name.contains("::") {
            return None;
        }
        let segments: Vec<&str> = name.split("::").filter(|s| !s.is_empty()).collect();
        if segments.len() < 2 {
            return None;
        }
        let ctx = ImportContext {
            index: self.index,
            state,
            path: state.path.clone(),
            language: state.language,
            diagnostics: Vec::new(),
            edges: Vec::new(),
            package_pairs: Default::default(),
        };
        for split in (1..segments.len()).rev() {
            let prefix = segments[..split].join("::");
            let Some(module) =
                crate::resolve::resolve_modules::rust::resolve_module_path(&ctx, &prefix)
            else {
                continue;
            };
            let mut current = module;
            let mut failed = false;
            for segment in &segments[split..segments.len() - 1] {
                match self.descend_rust_module(&ctx, &current, segment) {
                    Some(next) => current = next,
                    None => {
                        failed = true;
                        break;
                    }
                }
            }
            if failed {
                continue;
            }
            let last = segments[segments.len() - 1];
            let allow_private = ctx.allow_private(&current);
            let resolution = ctx.resolve_module_name(&current, last, allow_private, None);
            let resolution = match resolution {
                Resolution::Unresolved { .. } => self
                    .rust_binding_resolution(&ctx, &current, last, want)
                    .unwrap_or(resolution),
                other => other,
            };
            match resolution {
                Resolution::Unresolved { .. } => continue,
                Resolution::Internal(symbol) if !want.accepts(&symbol) => continue,
                other => return Some(other),
            }
        }
        // `path::Type::Variant` in type position uses the enum type itself.
        // Type-position only and CamelCase segments only: variants/types
        // are uppercase while methods are snake_case, so `Type::method`
        // paths must not collapse onto the parent type. For callables an
        // unmatched member must stay unresolved regardless.
        if matches!(want, Want::Type)
            && segments.len() >= 3
            && segments[segments.len() - 1]
                .chars()
                .next()
                .is_some_and(|c| c.is_uppercase())
        {
            let parent = segments[..segments.len() - 1].join("::");
            if let Some(Resolution::Internal(symbol)) =
                self.resolve_rust_path(state, &parent, Want::Type)
            {
                if symbol.is_type() {
                    return Some(Resolution::Internal(symbol));
                }
            }
        }
        None
    }

    /// Follow an intermediate path segment through a file's import bindings
    /// (`pub use other_crate::some_module` lands as `Binding::Module`).
    fn descend_rust_module(
        &self,
        ctx: &ImportContext<'_>,
        module: &crate::index::ModuleRef,
        segment: &str,
    ) -> Option<crate::index::ModuleRef> {
        let crate::index::ModuleRef::File(path) = module else {
            return None;
        };
        let state = ctx.index.files.get(path)?;
        match state.bindings.get(segment) {
            Some(Binding::Module(next)) => Some(next.clone()),
            _ => None,
        }
    }

    /// Last-segment lookup through a file's bindings when the module's
    /// exports/top-level symbols miss — covers `pub use` aliases recorded
    /// as `Binding::Symbol`/`ExternalSymbol`.
    fn rust_binding_resolution(
        &self,
        ctx: &ImportContext<'_>,
        module: &crate::index::ModuleRef,
        name: &str,
        want: Want,
    ) -> Option<Resolution> {
        let crate::index::ModuleRef::File(path) = module else {
            return None;
        };
        let state = ctx.index.files.get(path)?;
        match state.bindings.get(name) {
            Some(Binding::Symbol(symbol)) if want.accepts(symbol) => {
                Some(Resolution::Internal(symbol.clone()))
            }
            Some(Binding::ExternalSymbol { node_id, name }) => Some(Resolution::ExternalNode {
                node_id: node_id.clone(),
                name: name.clone(),
            }),
            Some(Binding::Module(next)) => Some(Resolution::Module(next.clone())),
            _ => None,
        }
    }
}
