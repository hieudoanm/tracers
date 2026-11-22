use tree_sitter::Node as TsNode;

use tracers_core::NodeKind;

use crate::util::{SyntaxBuilder, child_field, is_upper_camel, named_children};

use super::super::calls::collect_signature_types;
use super::super::items::parse_item;
use super::super::types::{go_type_expr_name, go_type_name};
use super::super::visibility::go_visibility;

pub(crate) fn parse_method(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let receiver_type = child_field(node, "receiver")
        .and_then(|receiver| {
            named_children(receiver)
                .into_iter()
                .find(|child| child.kind() == "parameter_declaration")
        })
        .and_then(|parameter| child_field(parameter, "type"))
        .map(|type_node| go_type_name(builder, type_node))
        .unwrap_or_else(|| "receiver".to_string());
    let qualified = format!("{receiver_type}.{name}");
    let parent = builder.find_symbol_by_qualified(&receiver_type);
    let exported = is_upper_camel(&name);
    let id = builder.add_symbol_qualified(
        NodeKind::Method,
        name,
        qualified,
        parent,
        node,
        go_visibility(exported),
        exported,
    );
    builder.push(id);
    // Record the receiver name (`r` in `func (r *T) M()`) with its declared
    // type so `r.field.Method()` chains resolve through field types.
    if let Some(receiver_param) = child_field(node, "receiver").and_then(|receiver| {
        named_children(receiver)
            .into_iter()
            .find(|child| child.kind() == "parameter_declaration")
    }) {
        if let Some(type_node) = child_field(receiver_param, "type") {
            let type_name = go_type_expr_name(builder, type_node);
            if !type_name.is_empty() {
                let mut cursor = receiver_param.walk();
                for name_node in receiver_param.children_by_field_name("name", &mut cursor) {
                    builder.add_local(
                        builder.text(name_node).to_string(),
                        type_name.clone(),
                        receiver_param,
                    );
                }
            }
        }
    }
    collect_signature_types(builder, node);
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body);
    }
    builder.pop();
}
