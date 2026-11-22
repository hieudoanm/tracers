use tree_sitter::Node as TsNode;

use crate::util::{SyntaxBuilder, child_field, first_named, is_builtin_type, named_children};

pub(crate) fn collect_go_type_refs(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    match node.kind() {
        "type_identifier" => {
            let name = builder.text(node).to_string();
            // Single uppercase letters are almost always generic type
            // parameters (`[T any]`), which bind lexically rather than
            // referencing a package type.
            let is_type_param =
                name.len() == 1 && name.chars().next().is_some_and(|c| c.is_ascii_uppercase());
            if !is_builtin_type(&name) && !is_type_param {
                builder.add_type_ref(name, node);
            }
        }
        "qualified_type" => {
            // Keep the package qualifier (`models.AccountEvent`,
            // `jwt.RegisteredClaims`); resolution uses the qualifier to find
            // the import binding and falls back to the leaf name.
            let text = builder.text(node).to_string();
            let name = child_field(node, "name")
                .map(|name| builder.text(name).to_string())
                .unwrap_or_else(|| text.clone());
            if !is_builtin_type(&name) {
                builder.add_type_ref(text, node);
            }
        }
        _ => {
            for child in named_children(node) {
                collect_go_type_refs(builder, child);
            }
        }
    }
}

pub(crate) fn go_type_name(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> String {
    go_type_name_inner(builder, node, false)
}

/// Type expression text that keeps package qualifiers (`gocql.Session`,
/// `models.AccountEvent`) so local/field type bindings can resolve through
/// import bindings. `go_type_name` strips them for symbol naming.
pub(crate) fn go_type_expr_name(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> String {
    go_type_name_inner(builder, node, true)
}

fn go_type_name_inner(
    builder: &SyntaxBuilder<'_>,
    node: TsNode<'_>,
    keep_qualifier: bool,
) -> String {
    match node.kind() {
        "pointer_expression" | "pointer_type" | "parenthesized_type" | "generic_type" => {
            first_named(node)
                .map(|child| go_type_name_inner(builder, child, keep_qualifier))
                .unwrap_or_default()
        }
        "qualified_type" => {
            if keep_qualifier {
                builder.text(node).to_string()
            } else {
                child_field(node, "name")
                    .map(|name| builder.text(name).to_string())
                    .unwrap_or_else(|| builder.text(node).to_string())
            }
        }
        "type_identifier" | "identifier" => builder.text(node).to_string(),
        _ => builder.text(node).to_string(),
    }
}
