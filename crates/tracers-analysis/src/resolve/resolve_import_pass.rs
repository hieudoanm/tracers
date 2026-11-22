//! Import pass: resolves every import, fills per-file bindings and re-export
//! maps, and returns dependency edges.

use std::collections::{BTreeMap, BTreeSet};

use tracers_core::{Diagnostic, Language, ResolutionStatus};
use tracers_parser::ImportData;

use crate::index::{Binding, FileState, ModuleRef, RepositoryIndex};

use super::resolve_types::{ImportEdge, ImportPass, Resolution};

/// Context for resolving imports within a single file.
pub struct ImportContext<'a> {
    pub index: &'a RepositoryIndex,
    pub state: &'a FileState,
    pub path: String,
    pub language: Language,
    pub diagnostics: Vec<Diagnostic>,
    pub edges: Vec<ImportEdge>,
    pub package_pairs: BTreeSet<(String, String)>,
}

impl<'a> ImportContext<'a> {
    pub fn process(
        &mut self,
        import: &ImportData,
        bindings: &mut BTreeMap<String, Binding>,
        re_exports: &mut BTreeMap<String, String>,
    ) {
        if import.kind == tracers_parser::ImportKind::Es
            && import.is_export
            && import.module.is_empty()
        {
            // `export { localName as alias };`
            for item in &import.items {
                if item.name == "*" {
                    continue;
                }
                let resolution = self.resolve_local_export(&item.name);
                if let Resolution::Internal(symbol) = resolution {
                    re_exports.insert(
                        item.alias.clone().unwrap_or_else(|| item.name.clone()),
                        symbol.node_id,
                    );
                } else {
                    self.unresolved_export(import, &item.name, &resolution);
                }
            }
            return;
        }

        let module = match self.resolve_module(import) {
            Ok(module) => module,
            Err(outcome) => {
                self.report_import_failure(import, &outcome);
                return;
            }
        };

        let external = matches!(module, ModuleRef::External(_));
        self.edges.push(ImportEdge {
            from_path: self.path.clone(),
            module: module.clone(),
            site: import.range,
            status: if external {
                ResolutionStatus::ResolvedExternal
            } else {
                ResolutionStatus::ResolvedInternal
            },
        });
        self.record_package_dependency(&module);

        let allow_private = self.allow_private(&module);
        let module_name = match &module {
            ModuleRef::External(name) => Some(name.clone()),
            _ => None,
        };

        if import.is_export {
            // Rust `pub use path::Name` flattens to a module path with no
            // item list — the re-exported name is the last path segment
            // (or the `as` alias).
            if import.items.is_empty() && import.kind == tracers_parser::ImportKind::Rust {
                let name = import.alias.clone().or_else(|| {
                    import
                        .module
                        .rsplit("::")
                        .next()
                        .filter(|s| !s.is_empty())
                        .map(str::to_string)
                });
                if let Some(name) = name {
                    if name == "*" {
                        if let ModuleRef::File(path) = &module {
                            if let Some(state) = self.index.files.get(path) {
                                for (export_name, ids) in &state.exports {
                                    for id in ids {
                                        re_exports.insert(export_name.clone(), id.clone());
                                    }
                                }
                            }
                        }
                        return;
                    }
                    let resolution = self.resolve_module_name(
                        &module,
                        &name,
                        allow_private,
                        module_name.as_deref(),
                    );
                    // `pub use` also binds the name locally, so record a
                    // binding for every resolution kind — including module
                    // re-exports (`pub use other_crate::some_module`) that
                    // have no symbol node id for `re_exports`.
                    match &resolution {
                        Resolution::Internal(symbol) => {
                            re_exports.insert(name.clone(), symbol.node_id.clone());
                            bindings.insert(name.clone(), Binding::Symbol(symbol.clone()));
                        }
                        Resolution::External { qualified, .. } => {
                            bindings.insert(
                                name.clone(),
                                Binding::ExternalSymbol {
                                    node_id: tracers_core::external_id(self.language, qualified),
                                    name: name.clone(),
                                },
                            );
                        }
                        Resolution::ExternalNode { node_id, name: en } => {
                            bindings.insert(
                                name.clone(),
                                Binding::ExternalSymbol {
                                    node_id: node_id.clone(),
                                    name: en.clone(),
                                },
                            );
                        }
                        _ => {
                            // `pub use path::module` re-exports the module
                            // itself when no symbol of that name exists
                            // inside it.
                            if matches!(module, ModuleRef::File(_)) {
                                bindings.insert(name.clone(), Binding::Module(module.clone()));
                            }
                        }
                    }
                }
                return;
            }
            for item in &import.items {
                if item.name == "*" {
                    if let ModuleRef::File(path) = &module {
                        if let Some(state) = self.index.files.get(path) {
                            for (name, ids) in &state.exports {
                                for id in ids {
                                    re_exports.insert(name.clone(), id.clone());
                                }
                            }
                        }
                    }
                    continue;
                }
                let resolution = self.resolve_module_name(
                    &module,
                    &item.name,
                    allow_private,
                    module_name.as_deref(),
                );
                if let Resolution::Internal(symbol) = resolution {
                    re_exports.insert(
                        item.alias.clone().unwrap_or_else(|| item.name.clone()),
                        symbol.node_id,
                    );
                } else {
                    self.unresolved_export(import, &item.name, &resolution);
                }
            }
            return;
        }

        match import.kind {
            tracers_parser::ImportKind::Es => {
                self.bind_es(import, &module, bindings, allow_private)
            }
            tracers_parser::ImportKind::Python => {
                self.bind_python(import, &module, bindings, allow_private)
            }
            tracers_parser::ImportKind::Go => self.bind_go(import, &module, bindings),
            tracers_parser::ImportKind::Rust => {
                self.bind_rust(import, &module, bindings, allow_private)
            }
            tracers_parser::ImportKind::Kotlin => {
                self.bind_kotlin(import, &module, bindings, allow_private)
            }
            tracers_parser::ImportKind::Swift => {
                if let ModuleRef::External(name) = &module {
                    bindings.insert(
                        import.alias.clone().unwrap_or_else(|| name.clone()),
                        Binding::Module(ModuleRef::External(name.clone())),
                    );
                }
            }
        }
    }
}

/// Resolve every import, fill per-file bindings and re-export maps, and return
/// dependency edges. Modules are indexed before this pass, so no source file
/// needs to be read again.
pub fn resolve_imports(index: &mut RepositoryIndex) -> ImportPass {
    let paths: Vec<String> = index.files.keys().cloned().collect();
    let mut pass = ImportPass::default();
    // Two rounds: `pub use` re-exports land in `state.exports` during the
    // pass, so names re-exported by a file processed later would otherwise
    // be invisible to earlier files (`use chess_core::Position` → lib.rs
    // `pub use position::Position`). Round 1 only warms the export maps;
    // its edges/diagnostics are discarded.
    for round in 0..2 {
        if round == 1 {
            pass.edges.clear();
            pass.diagnostics.clear();
            pass.package_pairs.clear();
        }
        for path in &paths {
            let Some(state) = index.files.get(path) else {
                continue;
            };
            let imports = state.parsed.imports.clone();
            let mut context = ImportContext {
                index,
                state,
                path: path.clone(),
                language: state.language,
                diagnostics: Vec::new(),
                edges: Vec::new(),
                package_pairs: BTreeSet::new(),
            };
            let mut bindings = BTreeMap::new();
            let mut re_exports: BTreeMap<String, String> = BTreeMap::new();
            for import in &imports {
                context.process(import, &mut bindings, &mut re_exports);
            }
            pass.diagnostics.append(&mut context.diagnostics);
            pass.edges.append(&mut context.edges);
            pass.package_pairs.append(&mut context.package_pairs);
            if let Some(state) = index.files.get_mut(path) {
                state.bindings = bindings;
                for (name, node_id) in re_exports {
                    state.exports.entry(name).or_default().push(node_id);
                }
                for ids in state.exports.values_mut() {
                    ids.sort();
                    ids.dedup();
                }
            }
        }
    }
    pass.diagnostics
        .sort_by_key(|diagnostic| diagnostic.sort_key());
    pass.diagnostics.dedup();
    pass
}
