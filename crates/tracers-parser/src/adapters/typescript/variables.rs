use tracers_core::NodeKind;
use tree_sitter::Node as TsNode;

use crate::util::{SyntaxBuilder, child_field, is_upper_camel, named_children};

use super::items::parse_item;
use super::types::collect_type_refs;

pub(crate) fn parse_variable(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let is_const = node
        .parent()
        .map(|parent| builder.text(parent).trim_start().starts_with("const "))
        .unwrap_or(false);
    if name_node.kind() != "identifier" {
        // Destructuring: `const [data, setData] = useState()`,
        // `const { a, b: c } = obj` — each bound name is a real symbol so
        // calls like `setData(...)` can resolve to the binding.
        let kind = if is_const {
            NodeKind::Constant
        } else {
            NodeKind::Variable
        };
        let mut bindings = Vec::new();
        pattern_bindings(builder, name_node, &mut bindings);
        for (name, binding_node) in bindings {
            builder.add_symbol(kind, name, binding_node, None, exported);
        }
        if let Some(value) = child_field(node, "value") {
            parse_item(builder, value, false);
        }
        return;
    }
    let name = builder.text(name_node).to_string();
    let value = child_field(node, "value");
    let is_function = value
        .map(|value| matches!(value.kind(), "arrow_function" | "function_expression"))
        .unwrap_or(false);
    let kind = if is_function {
        NodeKind::Function
    } else if is_const {
        NodeKind::Constant
    } else {
        NodeKind::Variable
    };
    let id = builder.add_symbol(kind, name.clone(), node, None, exported);
    if let Some(type_node) = child_field(node, "type") {
        collect_type_refs(builder, type_node);
        let type_name = super::visibility::normalize_type_name(builder.text(type_node));
        builder.add_local(name.clone(), type_name, node);
    }
    if let Some(value) = value {
        if value.kind() == "new_expression" {
            if let Some(constructor) = child_field(value, "constructor") {
                let type_name = super::visibility::normalize_type_name(builder.text(constructor));
                builder.add_local(name.clone(), type_name, node);
            }
        } else if value.kind() == "call_expression" {
            if let Some(function) = child_field(value, "function") {
                let callee = builder.text(function);
                if is_upper_camel(callee) {
                    builder.add_local(name.clone(), callee.to_string(), node);
                }
            }
        }
        if is_function {
            builder.push(id);
            parse_item(builder, value, false);
            builder.pop();
        } else {
            parse_item(builder, value, false);
        }
    }
}

/// Collect the identifiers bound by a destructuring pattern. Object keys
/// (`property_identifier`) are skipped; only `identifier` leaves count.
fn pattern_bindings<'a>(
    builder: &SyntaxBuilder<'a>,
    node: TsNode<'a>,
    out: &mut Vec<(String, TsNode<'a>)>,
) {
    match node.kind() {
        "identifier" => out.push((builder.text(node).to_string(), node)),
        "assignment_pattern" => {
            if let Some(left) = child_field(node, "left") {
                pattern_bindings(builder, left, out);
            }
        }
        "pair_pattern" | "pair" | "object_assignment_pattern" => {
            if let Some(value) = child_field(node, "value") {
                pattern_bindings(builder, value, out);
            } else {
                for child in named_children(node) {
                    pattern_bindings(builder, child, out);
                }
            }
        }
        kind if kind.contains("pattern") || kind == "rest_element" || kind == "spread_element" => {
            for child in named_children(node) {
                pattern_bindings(builder, child, out);
            }
        }
        _ => {}
    }
}
