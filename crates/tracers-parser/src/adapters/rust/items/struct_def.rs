use tree_sitter::Node as TsNode;

use tracers_core::{NodeKind, Visibility};

use crate::util::tree_sitter::range;
use crate::util::{SyntaxBuilder, child_field, named_children};

use super::super::calls::collect_type_refs;
use super::super::types::{rust_type_name, rust_visibility};

pub(crate) fn parse_struct(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = rust_visibility(builder, node);
    let id = builder.add_symbol(
        NodeKind::Struct,
        name,
        node,
        visibility,
        exported || visibility == Some(Visibility::Public),
    );
    builder.push(id);
    if let Some(type_parameters) = child_field(node, "type_parameters") {
        collect_type_refs(builder, type_parameters);
    }
    if let Some(body) = child_field(node, "body") {
        for field in named_children(body) {
            if field.kind() != "field_declaration" {
                continue;
            }
            let field_name = child_field(field, "name")
                .map(|name| builder.text(name).to_string())
                .unwrap_or_else(|| "field".to_string());
            builder.add_symbol(
                NodeKind::Property,
                field_name.clone(),
                field,
                rust_visibility(builder, field),
                false,
            );
            let type_name =
                child_field(field, "type").map(|type_node| rust_type_name(builder, type_node));
            if let Some(type_node) = child_field(field, "type") {
                collect_type_refs(builder, type_node);
            }
            // Field types are recorded as locals scoped to the struct so
            // `self.field.method()` / `local.field.method()` receivers can
            // be resolved through the field's declared type.
            if let Some(type_name) = type_name.filter(|name| !name.is_empty()) {
                builder.locals.push(crate::syntax::LocalBinding {
                    scope: Some(id),
                    name: field_name,
                    type_name,
                    range: range(field),
                });
            }
        }
    }
    builder.pop();
}
