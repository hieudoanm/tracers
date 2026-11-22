use tracers_parser::ImportData;

use crate::index::ModuleRef;

use crate::resolve::resolve_import_pass::ImportContext;
use crate::resolve::resolve_types::Resolution;

pub(crate) fn resolve_rust(
    ctx: &ImportContext<'_>,
    import: &ImportData,
) -> std::result::Result<ModuleRef, Resolution> {
    let raw = import
        .module
        .as_str()
        .trim_end_matches('*')
        .trim_end_matches("::");
    if let Some(module) = resolve_module_path(ctx, raw) {
        return Ok(module);
    }
    let first = raw.split("::").next().unwrap_or(raw);
    if matches!(first, "crate" | "self" | "super") {
        return Err(Resolution::Unresolved {
            name: raw.to_string(),
        });
    }
    if raw.is_empty() {
        return Err(Resolution::Unresolved {
            name: raw.to_string(),
        });
    }
    Ok(ModuleRef::External(
        raw.split("::").next().unwrap_or(raw).to_string(),
    ))
}

pub(crate) fn resolve_module_path(ctx: &ImportContext<'_>, raw: &str) -> Option<ModuleRef> {
    let state = ctx.state;
    let mut segments: Vec<String> = raw
        .split("::")
        .filter(|segment| !segment.is_empty() && *segment != "*")
        .map(|segment| segment.to_string())
        .collect();
    if segments.is_empty() {
        return None;
    }
    let mut module_segments: Vec<String> = Vec::new();
    let first = segments.first()?.as_str();
    let explicit_root = matches!(first, "crate" | "self" | "super");
    let plain_ident = !explicit_root;
    match first {
        "crate" => {
            segments.remove(0);
        }
        "self" => {
            segments.remove(0);
            module_segments = rust_current_module(ctx, state);
        }
        "super" => {
            let mut hops = 0;
            while segments.first().is_some_and(|segment| segment == "super") {
                segments.remove(0);
                hops += 1;
            }
            module_segments = rust_current_module(ctx, state);
            for _ in 0..hops {
                module_segments.pop();
            }
        }
        _ => {}
    }
    // Candidate (crate_root, segments) pairs. Rust 2018 uniform paths let a
    // plain first segment resolve against the current module first
    // (`use tree_sitter::x` inside `util/mod.rs` means `util::tree_sitter`),
    // then the crate root, then — for known workspace crate names — that
    // crate's root with the leading segment consumed (`use chess_core::X`).
    let mut attempts: Vec<(String, Vec<String>, bool)> = Vec::new();
    if plain_ident && module_segments.is_empty() {
        let local = rust_current_module(ctx, state);
        if !local.is_empty() {
            let mut scoped = local.clone();
            scoped.extend(segments.iter().cloned());
            attempts.push((state.crate_root.clone().unwrap_or_default(), scoped, false));
        }
        if let Some(first) = segments.first() {
            if let Some(other_root) = ctx.index.rust_crates.get(first) {
                attempts.push((other_root.clone(), segments[1..].to_vec(), true));
            }
        }
    }
    attempts.push((
        state.crate_root.clone().unwrap_or_default(),
        segments.clone(),
        false,
    ));

    for (crate_root, segs, cross_crate) in attempts {
        let mut full = module_segments.clone();
        full.extend(segs.iter().cloned());
        if let Some(path) = rust_file_for(ctx, &crate_root, &full) {
            return Some(ModuleRef::File(path));
        }
        // The crate root itself (`lib.rs`) is a valid module target for
        // cross-crate paths and explicit `crate::`/`self::`/`super::` roots —
        // but a bare `use foo;` must not bind to our own root.
        let mut module = module_segments.clone();
        module.extend(segs[..segs.len().saturating_sub(1)].iter().cloned());
        if !segs.is_empty()
            && (segs.len() > 1 || cross_crate || explicit_root || !module.is_empty())
        {
            if let Some(path) = rust_file_for(ctx, &crate_root, &module) {
                return Some(ModuleRef::File(path));
            }
        }
    }
    None
}

pub(crate) fn rust_current_module(
    _ctx: &ImportContext<'_>,
    state: &crate::index::FileState,
) -> Vec<String> {
    state
        .module_qualified
        .strip_prefix("crate::")
        .unwrap_or("")
        .split("::")
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_string())
        .collect()
}

pub(crate) fn rust_file_for(
    ctx: &ImportContext<'_>,
    crate_root: &str,
    segments: &[String],
) -> Option<String> {
    let relative = segments.join("/");
    let mut candidates = Vec::new();
    let prefix = if crate_root.is_empty() || crate_root == "." {
        String::new()
    } else {
        format!("{crate_root}/")
    };
    if relative.is_empty() {
        candidates.push(format!("{prefix}src/lib.rs"));
        candidates.push(format!("{prefix}src/main.rs"));
        candidates.push(format!("{prefix}lib.rs"));
        candidates.push(format!("{prefix}main.rs"));
    } else {
        candidates.push(format!("{prefix}src/{relative}.rs"));
        candidates.push(format!("{prefix}src/{relative}/mod.rs"));
        candidates.push(format!("{prefix}{relative}.rs"));
        candidates.push(format!("{prefix}{relative}/mod.rs"));
    }
    candidates
        .into_iter()
        .find(|path| ctx.index.files.contains_key(path))
}

pub(crate) fn rust_module_tail(ctx: &ImportContext<'_>, path: &str) -> String {
    ctx.index
        .files
        .get(path)
        .map(|state| {
            state
                .module_qualified
                .rsplit("::")
                .next()
                .unwrap_or("")
                .to_string()
        })
        .unwrap_or_else(|| crate::index::stem(path).to_string())
}
