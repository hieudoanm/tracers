use tree_sitter::Node as TsNode;

use tracers_core::NodeKind;

use super::super::types::{collect_go_type_refs, go_type_expr_name, go_type_name};
use super::super::visibility::go_visibility;
use crate::syntax::{HeritageRelation, LocalBinding};
use crate::util::tree_sitter::range;
use crate::util::{SyntaxBuilder, child_field, is_upper_camel, named_children};

pub(crate) fn parse_struct(builder: &mut SyntaxBuilder<'_>, from: u32, node: TsNode<'_>) {
    for child in named_children(node) {
        if child.kind() != "field_declaration_list" {
            continue;
        }
        for field in named_children(child) {
            if field.kind() != "field_declaration" {
                continue;
            }
            let name = child_field(field, "name")
                .map(|name| builder.text(name).to_string())
                .unwrap_or_default();
            let type_node = child_field(field, "type");
            if name.is_empty() {
                if let Some(type_node) = type_node {
                    let embedded = go_type_name(builder, type_node);
                    if !embedded.is_empty() {
                        builder.add_heritage(from, embedded, HeritageRelation::Inherits, field);
                    }
                    collect_go_type_refs(builder, type_node);
                }
                continue;
            }
            let exported = is_upper_camel(&name);
            builder.add_symbol(
                NodeKind::Property,
                name.clone(),
                field,
                go_visibility(exported),
                exported,
            );
            if let Some(type_node) = type_node {
                collect_go_type_refs(builder, type_node);
                // Field types scoped to the struct let `r.field.Method()`
                // receivers resolve through the declared field type.
                let type_name = go_type_expr_name(builder, type_node);
                if !type_name.is_empty() {
                    builder.locals.push(LocalBinding {
                        scope: Some(from),
                        name,
                        type_name,
                        range: range(field),
                    });
                }
            }
        }
    }
}
