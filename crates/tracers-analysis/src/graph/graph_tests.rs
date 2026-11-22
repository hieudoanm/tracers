//! Tests for graph construction.

use super::*;
use crate::index::RepositoryIndex;
use crate::resolve::resolve_imports;
use tempfile::tempdir;
use tracers_core::{EdgeKind, Language, NodeKind, ResolutionStatus, file_id, symbol_id};
use tracers_parser::{ParsedFile, SyntaxParser};

fn parse(path: &str, language: Language, source: &str) -> ParsedFile {
    let mut parser = SyntaxParser::new().unwrap();
    parser.parse(path, language, source).unwrap()
}

#[test]
fn builds_file_and_symbol_nodes() {
    let dir = tempdir().unwrap();
    let parsed = vec![parse(
        "a.ts",
        Language::TypeScript,
        "export function foo() { return 1; }\n",
    )];
    let mut index = RepositoryIndex::build(dir.path(), parsed).unwrap();
    let import_pass = resolve_imports(&mut index);
    let builder = GraphBuilder::new();
    let result = builder.build_from_index(&index, &import_pass, &[]).unwrap();
    assert!(result.graph.node(&file_id("a.ts")).is_some());
    assert!(
        result
            .graph
            .node(&symbol_id(NodeKind::Function, "a.ts", "foo", 0))
            .is_some()
    );
    assert_eq!(result.graph.node_count(), 4); // repository, file, module, function
}

#[test]
fn builds_call_edge_between_files() {
    let dir = tempdir().unwrap();
    let parsed = vec![
        parse(
            "math.ts",
            Language::TypeScript,
            "export function twice(n: number): number { return n * 2; }\n",
        ),
        parse(
            "main.ts",
            Language::TypeScript,
            "import { twice } from './math';\nexport function run(): number { return twice(1); }\n",
        ),
    ];
    let mut index = RepositoryIndex::build(dir.path(), parsed).unwrap();
    let import_pass = resolve_imports(&mut index);
    let builder = GraphBuilder::new();
    let result = builder.build_from_index(&index, &import_pass, &[]).unwrap();
    let run_id = symbol_id(NodeKind::Function, "main.ts", "run", 0);
    let twice_id = symbol_id(NodeKind::Function, "math.ts", "twice", 0);
    let graph = &result.graph;
    let out_edges = graph.out_edges(&run_id);
    let edges: Vec<_> = out_edges
        .iter()
        .flat_map(std::iter::once)
        .filter(|e| e.kind == EdgeKind::Calls && e.target == twice_id)
        .collect();
    assert!(!edges.is_empty(), "expected a call edge run -> twice");
}

#[test]
fn records_unresolved_call() {
    let dir = tempdir().unwrap();
    let parsed = vec![parse(
        "a.ts",
        Language::TypeScript,
        "export function run() { return missing(); }\n",
    )];
    let mut index = RepositoryIndex::build(dir.path(), parsed).unwrap();
    let import_pass = resolve_imports(&mut index);
    let builder = GraphBuilder::new();
    let result = builder.build_from_index(&index, &import_pass, &[]).unwrap();
    let run_id = symbol_id(NodeKind::Function, "a.ts", "run", 0);
    let graph = &result.graph;
    let out_edges = graph.out_edges(&run_id);
    let edges: Vec<_> = out_edges
        .iter()
        .flat_map(std::iter::once)
        .filter(|e| e.kind == EdgeKind::Calls)
        .collect();
    assert!(!edges.is_empty());
    assert!(
        edges
            .iter()
            .any(|e| matches!(e.status, ResolutionStatus::Unresolved)),
        "expected an unresolved call edge"
    );
    // The unresolved target must exist as a node so the graph is target-complete.
    assert!(graph.node("unresolved:typescript:missing").is_some());
}

#[test]
fn unresolved_nodes_are_scoped_by_language() {
    let dir = tempdir().unwrap();
    let parsed = vec![
        parse(
            "a.ts",
            Language::TypeScript,
            "export function run() { return missing(); }\n",
        ),
        parse(
            "b.py",
            Language::Python,
            "def run():\n    return missing()\n",
        ),
    ];
    let mut index = RepositoryIndex::build(dir.path(), parsed).unwrap();
    let import_pass = resolve_imports(&mut index);
    let builder = GraphBuilder::new();
    let result = builder.build_from_index(&index, &import_pass, &[]).unwrap();
    let graph = &result.graph;
    assert!(graph.node("unresolved:typescript:missing").is_some());
    assert!(graph.node("unresolved:python:missing").is_some());
}

#[test]
fn multi_file_go_package_shares_one_package_node() {
    let dir = tempdir().unwrap();
    let parsed = vec![
        parse("pkg/a.go", Language::Go, "package pkg\n\nfunc Foo() {}\n"),
        parse("pkg/b.go", Language::Go, "package pkg\n\nfunc Bar() {}\n"),
    ];
    let mut index = RepositoryIndex::build(dir.path(), parsed).unwrap();
    let import_pass = resolve_imports(&mut index);
    let builder = GraphBuilder::new();
    // Previously failed with ConflictingNode: the package node carried
    // per-file metadata, so the second file in a package collided.
    let result = builder.build_from_index(&index, &import_pass, &[]).unwrap();
    let package_nodes: Vec<_> = result
        .graph
        .nodes()
        .iter()
        .filter(|n| n.kind == NodeKind::Package)
        .collect();
    assert_eq!(package_nodes.len(), 1);
    assert!(package_nodes[0].file.is_none());
}

#[test]
fn resolves_stdlib_calls_as_external() {
    let dir = tempdir().unwrap();
    let parsed = vec![parse(
        "a.rs",
        Language::Rust,
        "fn f() {\n    let x = Some(1).unwrap();\n    println!(\"{x}\");\n    let _ = x.expect(\"missing\");\n}\n",
    )];
    let mut index = RepositoryIndex::build(dir.path(), parsed).unwrap();
    let import_pass = resolve_imports(&mut index);
    let mut builder = GraphBuilder::new();
    builder.include_external = true;
    let result = builder.build_from_index(&index, &import_pass, &[]).unwrap();
    let graph = &result.graph;
    for name in ["unwrap", "expect", "println"] {
        let external = format!("external:rust:std::{name}");
        assert!(graph.node(&external).is_some(), "missing {external}");
        assert!(graph.node(&format!("unresolved:rust:{name}")).is_none());
    }
    assert!(graph.edges().iter().any(|edge| {
        edge.status == ResolutionStatus::ResolvedExternal
            && edge.target.starts_with("external:rust:std::")
    }));
}

#[test]
fn drops_calls_with_garbage_names() {
    let dir = tempdir().unwrap();
    let parsed = vec![parse(
        "a.py",
        Language::Python,
        "def f():\n    return foo()()\n",
    )];
    let mut index = RepositoryIndex::build(dir.path(), parsed).unwrap();
    let import_pass = resolve_imports(&mut index);
    let builder = GraphBuilder::new();
    let _ = builder.build_from_index(&index, &import_pass, &[]).unwrap();
    for state in index.files.values() {
        for call in &state.parsed.calls {
            assert!(
                call.name
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '$' || "?!=".contains(c)),
                "garbage call name: {:?}",
                call.name
            );
        }
    }
}

#[test]
fn rust_wildcard_reexport_chain_resolves() {
    let dir = tempdir().unwrap();
    assert!(std::fs::create_dir_all(dir.path().join("src/handlers")).is_ok());
    assert!(std::fs::create_dir_all(dir.path().join("src/models")).is_ok());
    let parsed = vec![
        parse(
            "src/lib.rs",
            Language::Rust,
            "pub mod handlers;\npub mod models;\n",
        ),
        parse(
            "src/models/error.rs",
            Language::Rust,
            "pub enum AppError {\n    Internal,\n}\n",
        ),
        parse(
            "src/models.rs",
            Language::Rust,
            "pub mod error;\npub use error::*;\n",
        ),
        parse(
            "src/handlers/mod.rs",
            Language::Rust,
            "use crate::models::*;\n",
        ),
        parse(
            "src/handlers/sub.rs",
            Language::Rust,
            "use super::*;\npub fn f() -> Result<(), AppError> {\n    Err(AppError::Internal)\n}\n",
        ),
    ];
    let mut index = RepositoryIndex::build(dir.path(), parsed).unwrap();
    let import_pass = resolve_imports(&mut index);
    let builder = GraphBuilder::new();
    let result = builder.build_from_index(&index, &import_pass, &[]).unwrap();
    let graph = &result.graph;
    let unresolved: Vec<_> = graph
        .edges()
        .iter()
        .filter(|edge| edge.status == ResolutionStatus::Unresolved)
        .collect();
    assert!(
        unresolved.is_empty(),
        "unexpected unresolved edges: {unresolved:?}"
    );
}
