use std::collections::BTreeMap;

use tracers_parser::ImportData;

use crate::index::{Binding, ModuleRef, stem};

use super::super::resolve_import_pass::ImportContext;

impl<'a> ImportContext<'a> {
    pub(crate) fn bind_go(
        &mut self,
        import: &ImportData,
        module: &ModuleRef,
        bindings: &mut BTreeMap<String, Binding>,
    ) {
        if let Some(alias) = &import.alias {
            if alias != "_" && alias != "." {
                bindings.insert(alias.clone(), Binding::Module(module.clone()));
            }
            return;
        }
        // External modules: the qualifier used in code is the package's
        // declared name, which is not recoverable from the import path —
        // `github.com/segmentio/kafka-go` declares `package kafka`,
        // `.../jwt/v5` declares `package jwt`. Bind the last non-version
        // path segment plus its `-`/`_` components as candidates; real
        // package names are almost always one of them.
        if let ModuleRef::External(module_name) = module {
            let mut segments: Vec<&str> = module_name.split('/').collect();
            while segments
                .last()
                .is_some_and(|segment| is_version_segment(segment))
            {
                segments.pop();
            }
            if let Some(last) = segments.last() {
                for candidate in std::iter::once(*last).chain(
                    last.split(['-', '_'])
                        .filter(|component| component.len() >= 3),
                ) {
                    if is_go_identifier(candidate) {
                        bindings.insert(candidate.to_string(), Binding::Module(module.clone()));
                    }
                }
            }
            return;
        }
        let local = import.alias.clone().unwrap_or_else(|| match module {
            ModuleRef::Package(package) => self
                .index
                .packages
                .get(package)
                .map(|package| package.name.clone())
                .unwrap_or_else(|| {
                    import
                        .module
                        .rsplit('/')
                        .next()
                        .unwrap_or(&import.module)
                        .to_string()
                }),
            ModuleRef::External(_) => unreachable!(),
            ModuleRef::File(path) => stem(path).to_string(),
        });
        if !local.is_empty() && local != "_" {
            bindings.insert(local, Binding::Module(module.clone()));
        }
    }
}

/// `v2`, `v10` — Go module version path elements that are never package names.
fn is_version_segment(segment: &str) -> bool {
    segment.len() >= 2
        && segment.starts_with('v')
        && segment[1..].chars().all(|c| c.is_ascii_digit())
}

fn is_go_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_alphabetic() || c == '_')
        && chars.all(|c| c.is_alphanumeric() || c == '_')
}
