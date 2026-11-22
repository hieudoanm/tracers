//! Lookup functions for framework signatures.
//!
//! These functions search the [`FRAMEWORK_SIGNATURES`] database for entries
//! matching imports, decorators, base classes, calls, and manifest
//! dependencies.

use crate::framework::signatures::all_signatures;
use crate::framework::types::FrameworkSignature;

/// Look up all signatures whose import prefixes match a given import path.
///
/// A prefix matches only at a package boundary: the import path must equal
/// the prefix, or be followed by `/` (a subpath import). Prefixes ending in
/// `/` (scoped package scopes like `@angular/`) also match any package within
/// that scope. This prevents `react-native` from matching the `react` prefix.
pub fn signatures_for_import(import_path: &str) -> Vec<&'static FrameworkSignature> {
    // PHP namespaces use `\` separators and may carry a leading `\`;
    // normalize both so prefix matching works like `/`-separated modules.
    let import_path = import_path.trim_start_matches('\\');
    all_signatures()
        .iter()
        .filter(|sig| {
            sig.import_prefixes.iter().any(|prefix| {
                if import_path == *prefix {
                    return true;
                }
                let Some(rest) = import_path.strip_prefix(prefix) else {
                    return false;
                };
                rest.is_empty()
                    || rest.starts_with('/')
                    || rest.starts_with('\\')
                    || prefix.ends_with('/')
                    || prefix.ends_with('\\')
            })
        })
        .collect()
}

/// Look up all signatures whose decorators contain a given name.
pub fn signatures_for_decorator(decorator: &str) -> Vec<&'static FrameworkSignature> {
    all_signatures()
        .iter()
        .filter(|sig| sig.decorators.iter().any(|d| d == &decorator))
        .collect()
}

/// Look up all signatures whose base classes contain a given name.
pub fn signatures_for_base_class(base: &str) -> Vec<&'static FrameworkSignature> {
    all_signatures()
        .iter()
        .filter(|sig| sig.base_classes.iter().any(|b| b == &base))
        .collect()
}

/// Look up all signatures whose calls contain a given name.
pub fn signatures_for_call(call: &str) -> Vec<&'static FrameworkSignature> {
    all_signatures()
        .iter()
        .filter(|sig| sig.calls.iter().any(|c| c == &call))
        .collect()
}

/// Look up all signatures whose manifest dependencies contain a given dep.
pub fn signatures_for_manifest_dep(dep: &str) -> Vec<&'static FrameworkSignature> {
    all_signatures()
        .iter()
        .filter(|sig| sig.manifest_deps.iter().any(|d| dep.contains(*d)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn php_namespace_imports_match_prefixes() {
        // PHP `use` statements keep `\` separators; prefixes must still match.
        let names: Vec<&str> = signatures_for_import("Illuminate\\Support\\Facades\\Route")
            .iter()
            .map(|sig| sig.name)
            .collect();
        assert!(names.contains(&"laravel"), "got {names:?}");

        let names: Vec<&str> = signatures_for_import("\\Cake\\Controller\\Component")
            .iter()
            .map(|sig| sig.name)
            .collect();
        assert!(names.contains(&"cakephp"), "got {names:?}");

        // A leading backslash and unrelated namespaces must not match.
        assert!(signatures_for_import("App\\Models\\User").is_empty());
    }

    #[test]
    fn prefix_matching_respects_package_boundaries() {
        // `react` must not match `preact` or `react-native`-style siblings.
        assert!(
            signatures_for_import("preact")
                .iter()
                .all(|s| s.name != "react")
        );
        assert!(
            signatures_for_import("@remix-run/react")
                .iter()
                .any(|s| s.name == "remix")
        );
    }
}
