use tree_sitter::Node as TsNode;

use crate::util::{SyntaxBuilder, named_children};

pub(crate) fn find_modifiers(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> String {
    named_children(node)
        .into_iter()
        .find(|child| child.kind() == "modifiers")
        .map(|modifiers| builder.text(modifiers).to_string())
        .unwrap_or_default()
}
