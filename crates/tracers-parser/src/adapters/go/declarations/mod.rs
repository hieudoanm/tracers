pub mod function;
pub mod interface;
pub mod method;
pub mod struct_;

pub(crate) use self::function::parse_function;
pub(crate) use self::method::parse_method;

use tree_sitter::Node as TsNode;

use tracers_core::NodeKind;

use crate::syntax::LocalBinding;
use crate::util::{SyntaxBuilder, child_field, first_named, is_upper_camel, named_children, range};

use super::imports::collect_specs_named;
use super::items::parse_item;
use super::types::{collect_go_type_refs, go_type_expr_name};
use super::visibility::go_visibility;

pub(crate) fn parse_type_spec(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let Some(type_node) = child_field(node, "type") else {
        return;
    };
    let kind = match type_node.kind() {
        "struct_type" => NodeKind::Struct,
        "interface_type" => NodeKind::Interface,
        _ => NodeKind::Type,
    };
    let exported = is_upper_camel(&name);
    let id = builder.add_symbol(kind, name.clone(), node, go_visibility(exported), exported);
    builder.push(id);
    match type_node.kind() {
        "struct_type" => struct_::parse_struct(builder, id, type_node),
        "interface_type" => interface::parse_interface_type(builder, id, type_node),
        _ => collect_go_type_refs(builder, type_node),
    }
    builder.pop();
}

pub(crate) fn parse_declaration(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let mut specs = Vec::new();
    collect_specs_named(node, &mut specs);
    for spec in specs {
        // `var a, b T` / `a, b := f()` declare multiple names per spec.
        let mut cursor = spec.walk();
        let mut name_nodes: Vec<TsNode<'_>> =
            spec.children_by_field_name("name", &mut cursor).collect();
        if name_nodes.is_empty() {
            if let Some(left) = child_field(spec, "left") {
                name_nodes = named_children(left)
                    .into_iter()
                    .filter(|child| child.kind() == "identifier")
                    .collect();
            }
        }
        let names: Vec<String> = name_nodes
            .iter()
            .map(|name| builder.text(*name).to_string())
            .filter(|name| !name.contains('.'))
            .collect();
        if names.is_empty() {
            continue;
        }
        let kind = if spec.kind() == "const_spec" {
            NodeKind::Constant
        } else {
            NodeKind::Variable
        };
        let explicit_type = child_field(spec, "type")
            .map(|type_node| go_type_expr_name(builder, type_node))
            .filter(|name| !name.is_empty());
        // Right-hand expressions: `a, b := f()` shares one call across
        // names (positional results); `a, b := x, y` pairs them.
        let value_node = child_field(spec, "right").or_else(|| child_field(spec, "value"));
        let exprs: Vec<TsNode<'_>> = value_node
            .map(|value| {
                if value.kind() == "expression_list" {
                    named_children(value)
                } else {
                    vec![value]
                }
            })
            .unwrap_or_default();
        for (index, name) in names.iter().enumerate() {
            let exported = is_upper_camel(name);
            builder.add_symbol(
                kind,
                name.clone(),
                spec,
                go_visibility(exported),
                exported && builder.caller().is_none(),
            );
            let type_name = explicit_type
                .clone()
                .or_else(|| infer_initializer_type(builder, &exprs, index, names.len()));
            if let Some(type_name) = type_name {
                builder.locals.push(LocalBinding {
                    scope: builder.caller(),
                    name: name.clone(),
                    type_name,
                    range: range(spec),
                });
            }
        }
        if let Some(right) = child_field(spec, "right") {
            for child in named_children(right) {
                parse_item(builder, child);
            }
        }
        if let Some(value) = child_field(spec, "value") {
            parse_item(builder, value);
        }
    }
}

/// Infer a declared name's type from its initializer expression.
/// `exprs` are the right-hand side expressions; `index`/`name_count`
/// describe this name's position for multi-valued calls.
fn infer_initializer_type(
    builder: &SyntaxBuilder<'_>,
    exprs: &[TsNode<'_>],
    index: usize,
    name_count: usize,
) -> Option<String> {
    let expr = if exprs.len() == name_count {
        exprs.get(index).copied()
    } else if exprs.len() == 1 {
        exprs.first().copied()
    } else {
        None
    }?;
    let mut value = expr;
    // `x := &T{...}` / `x := *T{...}` wrap the literal.
    while matches!(value.kind(), "unary_expression" | "pointer_expression") {
        value = first_named(value)?;
    }
    match value.kind() {
        "composite_literal" => child_field(value, "type")
            .map(|type_node| go_type_expr_name(builder, type_node))
            .filter(|name| !name.is_empty()),
        // `x := f()` / `a, b := f()`: record a sentinel the analysis
        // resolves through the callee's declared result types.
        "call_expression" => {
            let function = child_field(value, "function")?;
            let callee = builder.text(function);
            if callee.is_empty() || callee.len() > 200 {
                return None;
            }
            // A lone call covers every name positionally; paired
            // expressions always start at result index 0.
            let result_index = if exprs.len() == 1 { index } else { 0 };
            Some(format!("@call:{callee}:{result_index}"))
        }
        _ => None,
    }
}
