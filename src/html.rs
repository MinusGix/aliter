use crate::{parse_node::ParseNode, Options};

#[derive(Debug, Clone, Copy)]
pub(crate) enum RealGroup {
    True,
    False,
    Root,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DomType {
    MOrd,
    MOp,
    MBin,
    MRel,
    MOpen,
    MClose,
    MPunct,
    MInner,
}

pub(crate) fn build_html(mut tree: Vec<ParseNode>, options: Options) {
    // Strip off any outer tag wrapper
    let (tag, tree) = if tree.len() == 1 && matches!(tree[0], ParseNode::Tag(_)) {
        if let ParseNode::Tag(tag) = tree.remove(0) {
            (Some(tag.tag), tag.body)
        } else {
            unreachable!()
        }
    } else {
        (None, tree)
    };

    let expression = build_expression(tree, &options, RealGroup::Root, (None, None));

    todo!()
}

pub(crate) fn build_expression(
    expression: Vec<ParseNode>,
    options: &Options,
    real_group: RealGroup,
    surrounding: (Option<DomType>, Option<DomType>),
) {
    // let mut groups = Vec::new();
    for expr in expression {}

    todo!()
}

pub(crate) fn build_group(
    group: Option<ParseNode>,
    options: &Options,
    base_options: Option<&Options>,
) {
    let group = if let Some(group) = group {
        group
    } else {
        todo!()
        // return make_span();
    };

    todo!()
}
