//! External dependency extraction from package manifests.
//!
//! Produces the set of declared dependency names/paths so path-qualified
//! calls like `serde_json::json!` or `fmt.Println` can resolve to
//! `resolved_external` edges instead of `unresolved`.

use std::collections::BTreeSet;

use crate::framework::manifest::ManifestContents;

/// Extract declared dependency names from discovered manifest files.
///
/// Names are stored both verbatim (full module paths, group ids) and as
/// their final segment where applicable, since source code refers to
/// packages by import alias rather than manifest coordinates.
pub(crate) fn extract_external_deps(manifests: &ManifestContents) -> BTreeSet<String> {
    let mut deps = BTreeSet::new();
    for (path, text) in &manifests.files {
        let name = path.rsplit('/').next().unwrap_or(path);
        match name {
            "Cargo.toml" => cargo_toml(text, &mut deps),
            "go.mod" => go_mod(text, &mut deps),
            "package.json" | "composer.json" => json_manifest(text, &mut deps),
            "Gemfile" => gemfile(text, &mut deps),
            "pubspec.yaml" => pubspec(text, &mut deps),
            "build.gradle" | "build.gradle.kts" => gradle(text, &mut deps),
            "pom.xml" => pom(text, &mut deps),
            "Package.swift" => swift_pm(text, &mut deps),
            "build.sbt" => sbt(text, &mut deps),
            "Podfile" => podfile(text, &mut deps),
            "Cartfile" => cartfile(text, &mut deps),
            _ if name.starts_with("requirements") && name.ends_with(".txt") => {
                requirements(text, &mut deps)
            }
            _ if name == "pyproject.toml" => pyproject(text, &mut deps),
            _ if name.ends_with(".csproj") => csproj(text, &mut deps),
            _ => {}
        }
    }
    deps
}

fn add(deps: &mut BTreeSet<String>, raw: &str) {
    let raw = raw.trim().trim_matches(|c| c == '"' || c == '\'');
    if raw.is_empty() || raw.starts_with('#') || raw.starts_with("//") {
        return;
    }
    // Strip version constraints / extras.
    let name = raw
        .split(['=', '<', '>', '~', '^', ' ', '\t', '(', '['])
        .next()
        .unwrap_or("")
        .trim();
    if name.is_empty() {
        return;
    }
    deps.insert(name.to_string());
    // Also index the final path segment: `github.com/x/y` is imported as `y`.
    if let Some(last) = name.rsplit('/').next() {
        deps.insert(last.to_string());
    }
    // Gradle `group:artifact:version` → artifact.
    if let Some(artifact) = name.split(':').nth(1) {
        if !artifact.is_empty() {
            deps.insert(artifact.to_string());
        }
    }
}

fn cargo_toml(text: &str, deps: &mut BTreeSet<String>) {
    let mut in_deps = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_deps = line.contains("dependencies");
            continue;
        }
        if !in_deps || line.starts_with('#') {
            continue;
        }
        let key = line.split('=').next().unwrap_or("").trim();
        let key = key.trim_matches('"');
        if !key.is_empty() && key != "workspace" {
            deps.insert(key.to_string());
        }
    }
}

fn go_mod(text: &str, deps: &mut BTreeSet<String>) {
    let mut in_require = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("require (") {
            in_require = true;
            continue;
        }
        if in_require && line == ")" {
            in_require = false;
            continue;
        }
        if line.starts_with("require ") && !line.contains('(') {
            let rest = line.trim_start_matches("require ");
            if let Some(path) = rest.split_whitespace().next() {
                add(deps, path);
            }
            continue;
        }
        if in_require && !line.is_empty() && !line.starts_with("//") {
            if let Some(path) = line.split_whitespace().next() {
                add(deps, path);
            }
        }
    }
}

fn json_manifest(text: &str, deps: &mut BTreeSet<String>) {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return;
    };
    for section in [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
        "require",
        "require-dev",
    ] {
        if let Some(obj) = value.get(section).and_then(|v| v.as_object()) {
            for key in obj.keys() {
                deps.insert(key.clone());
            }
        }
    }
}

fn gemfile(text: &str, deps: &mut BTreeSet<String>) {
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("gem") {
            let rest = rest.trim_start();
            if let Some(first) = rest.split([',', ' ', '\t']).next() {
                add(deps, first);
            }
        }
    }
}

fn pubspec(text: &str, deps: &mut BTreeSet<String>) {
    let mut in_deps = false;
    for line in text.lines() {
        let trimmed = line.trim_end();
        if trimmed.ends_with(':') && !trimmed.starts_with(' ') {
            in_deps = matches!(trimmed, "dependencies:" | "dev_dependencies:");
            continue;
        }
        if in_deps && line.starts_with(' ') && !line.trim().is_empty() {
            if let Some((key, _)) = line.trim().split_once(':') {
                deps.insert(key.trim().to_string());
            }
        }
    }
}

fn gradle(text: &str, deps: &mut BTreeSet<String>) {
    for line in text.lines() {
        let line = line.trim();
        for config in [
            "implementation",
            "api",
            "compileOnly",
            "runtimeOnly",
            "testImplementation",
            "kapt",
            "annotationProcessor",
        ] {
            if let Some(idx) = line.find(&format!("{config}(")) {
                let rest = &line[idx + config.len() + 1..];
                if let Some(first) = rest.split([',', ')']).next() {
                    let first = first.trim().trim_matches('"').trim_matches('\'');
                    add(deps, first);
                }
                break;
            }
        }
    }
}

fn pom(text: &str, deps: &mut BTreeSet<String>) {
    for cap in text.split("<dependency>") {
        let mut artifact = None;
        for tag in ["<artifactId>", "<groupId>"] {
            if let Some(start) = cap.find(tag) {
                let rest = &cap[start + tag.len()..];
                if let Some(end) = rest.find('<') {
                    let value = rest[..end].trim();
                    if tag == "<artifactId>" {
                        artifact = Some(value.to_string());
                    } else if let Some(a) = &artifact {
                        deps.insert(format!("{value}:{a}"));
                    }
                }
            }
        }
        if let Some(a) = artifact {
            deps.insert(a);
        }
    }
}

fn swift_pm(text: &str, deps: &mut BTreeSet<String>) {
    for line in text.lines() {
        if let Some(idx) = line.find(".package(url:") {
            let rest = &line[idx..];
            if let Some(q1) = rest.find('"') {
                let after = &rest[q1 + 1..];
                if let Some(q2) = after.find('"') {
                    let url = &after[..q2];
                    let last = url.trim_end_matches('/').rsplit('/').next().unwrap_or("");
                    let last = last.strip_suffix(".git").unwrap_or(last);
                    if !last.is_empty() {
                        deps.insert(last.to_string());
                    }
                }
            }
        }
    }
}

fn sbt(text: &str, deps: &mut BTreeSet<String>) {
    for line in text.lines() {
        // "group" % "artifact" % "version"
        let parts: Vec<&str> = line.split('%').collect();
        if parts.len() >= 2 {
            let artifact = parts[1].trim().trim_matches('"');
            if !artifact.is_empty() {
                deps.insert(artifact.to_string());
            }
        }
    }
}

fn podfile(text: &str, deps: &mut BTreeSet<String>) {
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("pod") {
            if let Some(first) = rest.split([',', ' ', '\t']).next() {
                add(deps, first);
            }
        }
    }
}

fn cartfile(text: &str, deps: &mut BTreeSet<String>) {
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("github") {
            if let Some(first) = rest.split([',', ' ', '\t']).next() {
                let first = first.trim_matches('"').trim_matches('\'');
                if let Some(last) = first.rsplit('/').next() {
                    deps.insert(last.to_string());
                }
            }
        }
    }
}

fn requirements(text: &str, deps: &mut BTreeSet<String>) {
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with("-e")
            || line.starts_with("git+")
        {
            continue;
        }
        add(deps, line);
    }
}

fn pyproject(text: &str, deps: &mut BTreeSet<String>) {
    // Covers [project] dependencies arrays and [tool.poetry.dependencies].
    let mut in_poetry = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_poetry = trimmed.contains("poetry") && trimmed.contains("dependencies");
        }
        if in_poetry {
            if let Some((key, _)) = trimmed.split_once('=') {
                let key = key.trim().trim_matches('"');
                if !key.is_empty() && key != "python" {
                    deps.insert(key.to_string());
                }
            }
        } else if let Some(stripped) = trimmed.strip_prefix('"') {
            // Quoted requirement inside a dependencies array.
            if let Some(end) = stripped.find('"') {
                add(deps, &stripped[..end]);
            }
        }
    }
}

fn csproj(text: &str, deps: &mut BTreeSet<String>) {
    for cap in text.split("<PackageReference") {
        for attr in ["Include=\"", "Update=\""] {
            if let Some(idx) = cap.find(attr) {
                let rest = &cap[idx + attr.len()..];
                if let Some(end) = rest.find('"') {
                    deps.insert(rest[..end].to_string());
                }
            }
        }
    }
}
