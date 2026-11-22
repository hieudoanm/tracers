use tree_sitter::Node as TsNode;

use crate::syntax::HeritageRelation;
use tracers_core::{NodeKind, Visibility};

use crate::util::{SyntaxBuilder, child_field, named_children};

use super::super::calls::collect_type_refs;
use super::super::function::collect_function_signature;
use super::super::types::{rust_type_name, rust_visibility};

pub(crate) fn parse_impl(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let type_node = child_field(node, "type");
    let type_name = type_node
        .map(|type_node| rust_type_name(builder, type_node))
        .unwrap_or_default();
    if type_name.is_empty() {
        return;
    }
    // The impl header gives a bare or path-qualified type name (`Parser`,
    // `crate::x::Parser`); type symbols carry module-prefixed qualified
    // names, so fall back to a leaf-name match on type-kind symbols.
    let type_local = builder.find_symbol_by_qualified(&type_name).or_else(|| {
        let leaf = type_name.rsplit("::").next().unwrap_or(type_name.as_str());
        builder
            .symbols
            .iter()
            .filter(|symbol| {
                symbol.name == leaf
                    && matches!(
                        symbol.kind,
                        NodeKind::Class
                            | NodeKind::Struct
                            | NodeKind::Enum
                            | NodeKind::Interface
                            | NodeKind::Trait
                            | NodeKind::Type
                    )
            })
            .map(|symbol| symbol.local_id)
            .next()
    });
    // Method qualified names must share the type symbol's qualified prefix
    // (`crate::parser.Parser.bump`) so parent-type lookup and
    // `Type.member` qualified keys line up.
    let type_qualified = type_local.and_then(|local| {
        builder
            .symbols
            .iter()
            .find(|symbol| symbol.local_id == local)
            .map(|symbol| symbol.qualified_name.clone())
    });
    if let Some(trait_node) = child_field(node, "trait") {
        let trait_name = rust_type_name(builder, trait_node);
        if let Some(type_local) = type_local {
            builder.add_heritage(
                type_local,
                trait_name,
                HeritageRelation::Implements,
                trait_node,
            );
        }
    }
    if let Some(type_node) = type_node {
        collect_type_refs(builder, type_node);
    }
    if let Some(body) = child_field(node, "body") {
        for item in named_children(body) {
            if !matches!(item.kind(), "function_item" | "function_signature_item") {
                super::parse_item(builder, item, false);
                continue;
            }
            let Some(name_node) = child_field(item, "name") else {
                continue;
            };
            let name = builder.text(name_node).to_string();
            let visibility = rust_visibility(builder, item);
            let qualified = match &type_qualified {
                Some(prefix) => format!("{prefix}.{name}"),
                None => format!("{type_name}.{name}"),
            };
            let id = builder.add_symbol_qualified(
                NodeKind::Method,
                name,
                qualified,
                type_local,
                item,
                visibility,
                visibility == Some(Visibility::Public),
            );
            builder.push(id);
            collect_function_signature(builder, item);
            if let Some(function_body) = child_field(item, "body") {
                super::parse_item(builder, function_body, false);
            }
            builder.pop();
        }
    }
}
