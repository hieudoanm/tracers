use crate::syntax::{ImportData, ImportItem, ImportKind, ParsedFile};
use crate::util::SyntaxBuilder;
use tracers_core::Language;
use tracers_core::{Position, SourceRange};
use tree_sitter::Tree;

pub fn parse(language: Language, path: &str, source: &str, _tree: &Tree) -> ParsedFile {
    parse_source(language, path, source)
}

/// Markdown has no tree-sitter grammar; the adapter scans link targets
/// directly from the source text.
///
/// Only repository-relative link targets are recorded as imports. External
/// URLs, `mailto:`/`data:` schemes, pure `#anchors`, site-absolute paths,
/// and images (`![alt](src)`) are documentation, not dependencies, and are
/// skipped so they don't pollute the graph or diagnostics.
pub fn parse_source(language: Language, path: &str, source: &str) -> ParsedFile {
    let mut builder = SyntaxBuilder::new(source, language, path);
    let mut i = 0;
    // `i` must always be a char boundary: every jump target below lands on
    // an ASCII delimiter, and the fallback step advances a whole char.
    while i < source.len() {
        if source[i..].starts_with("[[") {
            if let Some(end) = source[i..].find("]]") {
                if let Some(target) = clean_wiki_target(&source[i + 2..i + end]) {
                    push_import(&mut builder, &target, i, i + 2);
                }
                i = i + end + 2;
                continue;
            }
        }
        if source[i..].starts_with('[') && !(i > 0 && source.as_bytes()[i - 1] == b'!') {
            if let Some(close) = source[i..].find("](") {
                let j = i + close + 2;
                if let Some(end) = source[j..].find(')') {
                    if let Some(target) = clean_link_target(&source[j..j + end]) {
                        push_import(&mut builder, &target, i, i + 1);
                    }
                    i = j + end + 1;
                    continue;
                }
            }
        }
        i += source[i..].chars().next().map_or(1, char::len_utf8);
    }
    builder.finish()
}

fn push_import(builder: &mut SyntaxBuilder<'_>, target: &str, start: usize, name_end: usize) {
    let range = SourceRange::new(
        Position {
            line: 1,
            column: 1,
            byte: start as u32,
        },
        Position {
            line: 1,
            column: 2,
            byte: name_end as u32,
        },
    );
    builder.imports.push(ImportData {
        module: target.to_string(),
        alias: None,
        items: vec![ImportItem {
            name: target.to_string(),
            alias: None,
        }],
        kind: ImportKind::Es,
        range,
        is_export: false,
    });
}

/// Normalize a markdown link target into a repository-relative specifier,
/// or `None` when the target is not a repo dependency (external URL,
/// anchor, site-absolute path, empty).
fn clean_link_target(raw: &str) -> Option<String> {
    let mut target = raw.trim();
    // `<path with spaces>` form.
    if let Some(inner) = target.strip_prefix('<') {
        target = inner.split('>').next().unwrap_or("").trim();
    }
    // Optional `"title"` after the destination.
    if let Some(idx) = target.find(char::is_whitespace) {
        target = &target[..idx];
    }
    // Strip `#fragment` and `?query` — they don't affect file identity.
    if let Some(idx) = target.find(['#', '?']) {
        target = &target[..idx];
    }
    let target = target.trim();
    if target.is_empty()
        || target.starts_with('#')
        || target.starts_with('/')
        || has_uri_scheme(target)
    {
        return None;
    }
    // Bare names (`[[Page]]`, `[x](guide.md)`) become `./`-relative so module
    // resolution treats them as repository files rather than externals.
    if !target.starts_with('.') {
        Some(format!("./{target}"))
    } else {
        Some(target.to_string())
    }
}

/// `[[Wiki Link]]` targets are note names, not file paths — keep them bare
/// so they resolve as external references rather than unresolved imports.
fn clean_wiki_target(raw: &str) -> Option<String> {
    let mut target = raw.trim();
    // `[[path/Note|Alias]]` — drop the alias.
    if let Some(idx) = target.find('|') {
        target = &target[..idx];
    }
    if let Some(idx) = target.find(['#', '?']) {
        target = &target[..idx];
    }
    let target = target.trim();
    if target.is_empty() || has_uri_scheme(target) {
        return None;
    }
    Some(target.to_string())
}

/// `scheme:` prefix such as `https:`, `mailto:`, `data:`, `javascript:`.
fn has_uri_scheme(target: &str) -> bool {
    let Some(idx) = target.find(':') else {
        return false;
    };
    let scheme = &target[..idx];
    !scheme.is_empty()
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
        && scheme
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
}
