use crate::{
    build_common::{make_span, make_span_s},
    dom_tree::{CssStyle, DomSpan, WithHtmlDomNode},
    functions,
    parse_node::ParseNode,
    tree::ClassList,
    Options,
};

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
        let ParseNode::Tag(tag) = tree.into_iter().nth(0).unwrap() else {
            unreachable!()
        };

        (Some(tag.tag), tag.body)
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
    // for expr in expression {
    //     let output = build_group(Some(&expr), options, None);
    // }

    todo!()
}

pub(crate) fn build_group(
    group: Option<&ParseNode>,
    options: &Options,
    base_options: Option<&Options>,
) -> Box<dyn WithHtmlDomNode> {
    let Some(group) = group else {
        return Box::new(make_span::<Box<dyn WithHtmlDomNode>>(
            ClassList::new(),
            Vec::new(),
            None,
            CssStyle::default(),
        ));
    };

    if let Some(html_builder) = functions::FUNCTIONS.find_html_builder_for_type(group.typ()) {
        let group_node = html_builder(group, options);

        // If the size changed between the parent and the current group, account for that size
        // difference
        if let Some(base_options) = base_options {
            if options.size != base_options.size {
                let mut group_node = make_span(
                    options.sizing_classes(base_options),
                    vec![group_node],
                    Some(options),
                    CssStyle::default(),
                );

                let mult = options.size_multiplier() / base_options.size_multiplier();

                group_node.node.height *= mult;
                group_node.node.depth *= mult;

                return Box::new(group_node);
            }
        }

        group_node
    } else {
        panic!("Got group of unknown type");
    }
}

pub(crate) fn make_null_delimiter(options: &Options, classes: ClassList) -> DomSpan {
    let classes = options
        .base_sizing_classes()
        .into_iter()
        .chain(["nulldelimiter".to_string()])
        .chain(classes)
        .collect::<ClassList>();
    make_span_s(classes, Vec::new())
}
