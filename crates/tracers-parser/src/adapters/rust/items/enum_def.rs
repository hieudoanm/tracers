use tree_sitter::Node as TsNode;

use tracers_core::{NodeKind, Visibility};

use crate::util::{SyntaxBuilder, child_field, named_children};

use super::super::types::rust_visibility;

pub(crate) fn parse_enum(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = rust_visibility(builder, node);
    let enum_exported = exported || visibility == Some(Visibility::Public);
    let id = builder.add_symbol(NodeKind::Enum, name, node, visibility, enum_exported);
    builder.push(id);
    if let Some(body) = child_field(node, "body") {
        for variant in named_children(body) {
            if variant.kind() != "enum_variant" {
                continue;
            }
            if let Some(variant_name) = child_field(variant, "name") {
                builder.add_symbol(
                    NodeKind::Constant,
                    builder.text(variant_name).to_string(),
                    variant,
                    None,
                    enum_exported,
                );
            }
        }
    }
    builder.pop();
}
