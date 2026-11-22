use crate::resolve::{Resolution, resolve_resolver::Resolver};

use crate::index::{FileState, RepositoryIndex};
use tracers_core::{Edge, EdgeKind, ResolutionOrigin, ResolutionStatus, external_id};

use super::super::graph_helpers::unresolved_id;
use super::super::graph_types::GraphBuilder;

pub(crate) fn add_type_ref_edges(
    builder: &GraphBuilder,
    graph: &mut tracers_core::Graph,
    index: &RepositoryIndex,
    state: &FileState,
) -> crate::error::Result<()> {
    let resolver = Resolver::new(index);

    for type_ref in &state.parsed.type_refs {
        let from_id = type_ref
            .from
            .and_then(|local| state.symbol(local))
            .map(|symbol| symbol.node_id.clone())
            .unwrap_or_else(|| state.module_id.clone());

        let resolution = resolver.resolve_type_name(state, type_ref.from, &type_ref.name);
        match resolution {
            Resolution::Internal(symbol) => {
                graph.add_edge(
                    Edge::new(
                        from_id.clone(),
                        symbol.node_id,
                        EdgeKind::UsesType,
                        ResolutionStatus::ResolvedInternal,
                        ResolutionOrigin::Syntax,
                    )
                    .at(type_ref.range),
                )?;
            }
            Resolution::External { qualified, .. } => {
                let node_id = external_id(state.language, &qualified);
                if builder.include_external {
                    builder.add_external_node(graph, state.language, &qualified);
                    graph.add_edge(
                        Edge::new(
                            from_id.clone(),
                            node_id,
                            EdgeKind::UsesType,
                            ResolutionStatus::ResolvedExternal,
                            ResolutionOrigin::Syntax,
                        )
                        .at(type_ref.range),
                    )?;
                }
            }
            Resolution::Module(module) => {
                let target = builder.module_target_node(
                    index,
                    &crate::graph::graph_symbol_edges::module_id_str(&module),
                );
                graph.add_edge(
                    Edge::new(
                        from_id.clone(),
                        target,
                        EdgeKind::UsesType,
                        ResolutionStatus::ResolvedInternal,
                        ResolutionOrigin::Syntax,
                    )
                    .at(type_ref.range),
                )?;
            }
            _ => {
                if let Some(qualified) = crate::resolve::builtin_calls::builtin_type_qualified(
                    state.language,
                    &type_ref.name,
                ) {
                    let node_id = external_id(state.language, &qualified);
                    if builder.include_external {
                        builder.add_external_node(graph, state.language, &qualified);
                        graph.add_edge(
                            Edge::new(
                                from_id.clone(),
                                node_id,
                                EdgeKind::UsesType,
                                ResolutionStatus::ResolvedExternal,
                                ResolutionOrigin::Syntax,
                            )
                            .at(type_ref.range),
                        )?;
                    }
                } else if let Some(qualified) =
                    qualified_type_target(state.language, &type_ref.name, &index.external_deps)
                {
                    // Qualified type from a stdlib package or declared
                    // dependency (`time.Time`, `serde_json::Value`).
                    let node_id = external_id(state.language, &qualified);
                    if builder.include_external {
                        builder.add_external_node(graph, state.language, &qualified);
                        graph.add_edge(
                            Edge::new(
                                from_id.clone(),
                                node_id,
                                EdgeKind::UsesType,
                                ResolutionStatus::ResolvedExternal,
                                ResolutionOrigin::Syntax,
                            )
                            .at(type_ref.range),
                        )?;
                    }
                } else if tracers_parser::util::builtins::is_builtin_type(&type_ref.name) {
                    // Primitive/builtin type names with no stdlib mapping
                    // (e.g. `int` in C, `i32` in Rust) carry no dependency
                    // information — skip rather than emit unresolved noise.
                    continue;
                } else {
                    builder.add_unresolved_node(graph, state.language, &type_ref.name);
                    graph.add_edge(
                        Edge::new(
                            from_id.clone(),
                            unresolved_id(state.language, &type_ref.name),
                            EdgeKind::UsesType,
                            ResolutionStatus::Unresolved,
                            ResolutionOrigin::Syntax,
                        )
                        .at(type_ref.range),
                    )?;
                }
            }
        }
    }
    Ok(())
}

/// Split a qualified type name (`serde_json::Value`, `time.Time`,
/// `os.path.PathLike`) into qualifier/member and check it against stdlib
/// packages and manifest dependencies.
fn qualified_type_target(
    language: tracers_core::Language,
    name: &str,
    external_deps: &std::collections::BTreeSet<String>,
) -> Option<String> {
    // Split at the last `::` or `.` separator, whichever comes last.
    let idx = name
        .rfind("::")
        .map(|i| (i, 2))
        .into_iter()
        .chain(name.rfind('.').map(|i| (i, 1)))
        .max_by_key(|(i, _)| *i)?;
    let (i, sep_len) = idx;
    let (receiver, member) = name.split_at(i);
    let member = &member[sep_len..];
    if receiver.is_empty() || member.is_empty() {
        return None;
    }
    crate::resolve::builtin_calls::receiver_external_target(
        language,
        receiver,
        member,
        external_deps,
    )
}
