use crate::{
    build_common::{self, make_empty_span, make_span, make_span_s},
    dom_tree::{CssStyle, HtmlDomNode, HtmlNode, Span, WithHtmlDomNode},
    functions,
    parse_node::ParseNode,
    spacing_data::{SPACINGS, TIGHT_SPACINGS},
    tree::ClassList,
    unit::Measurement,
    util::find_assoc_data,
    Options,
};

// Binary atoms (first class `mbin`) change into ordinary atoms (`mord`)
// depending on their surroundings. See TeXbook pg. 442-446, Rules 5 and 6,
// and the text before Rule 19.
const BIN_LEFT_CANCELLER: &'static [&'static str] =
    &["leftmost", "mbin", "mopen", "mrel", "mop", "mpunct"];
const BIN_RIGHT_CANCELLER: &'static [&'static str] = &["rightmost", "mrel", "mclose", "mpunct"];

enum Side {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
impl DomType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "mord" => Some(DomType::MOrd),
            "mop" => Some(DomType::MOp),
            "mbin" => Some(DomType::MBin),
            "mrel" => Some(DomType::MRel),
            "mopen" => Some(DomType::MOpen),
            "mclose" => Some(DomType::MClose),
            "mpunct" => Some(DomType::MPunct),
            "minner" => Some(DomType::MInner),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            DomType::MOrd => "mord",
            DomType::MOp => "mop",
            DomType::MBin => "mbin",
            DomType::MRel => "mrel",
            DomType::MOpen => "mopen",
            DomType::MClose => "mclose",
            DomType::MPunct => "mpunct",
            DomType::MInner => "minner",
        }
    }
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
) -> Vec<HtmlNode> {
    let mut groups = Vec::new();
    for expr in &expression {
        let output = build_group(Some(&expr), options, None);
        match output {
            HtmlNode::DocumentFragment(frag) => {
                let children = frag.children;
                groups.extend(children);
            }
            _ => groups.push(output),
        }
    }

    // Combine consecutive `SymbolNode`s into a single `SymbolNode`.
    build_common::try_combine_chars(&mut groups);

    // If expression is a partial group, let the parent handle spacings to avoid processing groups
    // multiple times
    if real_group == RealGroup::False {
        return groups;
    }

    let glue_options = if expression.len() == 1 {
        let node = &expression[0];
        let opts = match node {
            ParseNode::Sizing(sizing) => options.having_size(sizing.size),
            ParseNode::Styling(styling) => options.having_style(styling.style.into_style_id()),
            _ => None,
        };

        // TODO: don't clone
        opts.unwrap_or_else(|| options.clone())
    } else {
        options.clone()
    };

    // Dummy spans for determining spacings between surrounding atoms.
    // If `expression` has no atoms on the left or right, class "leftmost" or "rightmost",
    // respectively, is used to indicate it
    let dummy_prev_class = if let Some(sur0) = &surrounding.0 {
        sur0.as_str()
    } else {
        "leftmost"
    };
    let dummy_prev: HtmlNode = make_span::<HtmlNode>(
        vec![dummy_prev_class.to_string()],
        Vec::new(),
        Some(options),
        CssStyle::default(),
    )
    .into();

    let dummy_next_class = if let Some(sur1) = &surrounding.1 {
        sur1.as_str()
    } else {
        "rightmost"
    };
    let dummy_next: HtmlNode = make_span::<HtmlNode>(
        vec![dummy_next_class.to_string()],
        Vec::new(),
        Some(options),
        CssStyle::default(),
    )
    .into();

    // TODO: These code assumes that a node's math class is the first element
    // of its `classes` array. A later cleanup should ensure this, for
    // instance by changing the signature of `makeSpan`.

    // Before determining what spaces to insert, perform bin cancellation.
    // Binary operators change to ordinary symbols in some contexts.
    let is_root = real_group == RealGroup::Root;
    traverse_non_space_nodes(
        &mut groups,
        &|node, prev| {
            let prev_type = prev.node().classes.get(0)?;
            let typ = node.node().classes.get(0)?;
            if prev_type == "mbin" && BIN_RIGHT_CANCELLER.contains(&typ.as_str()) {
                prev.node_mut().classes[0] = "mord".to_string();
            } else if typ == "mbin" && BIN_LEFT_CANCELLER.contains(&prev_type.as_str()) {
                node.node_mut().classes[0] = "mord".to_string();
            }

            None
        },
        dummy_prev.clone(),
        None,
        Some(dummy_next.clone()),
        is_root,
    );

    traverse_non_space_nodes(
        &mut groups,
        &|node, prev| {
            let prev_typ = get_type_of_dom_tree(Some(prev), None)?;
            let typ = get_type_of_dom_tree(Some(node), None)?;

            // 'mtight' indicates that the node is script or scriptscript style
            let spacings = if node.node().classes.iter().any(|c| c == "mtight") {
                find_assoc_data(TIGHT_SPACINGS, (prev_typ, typ))
            } else {
                find_assoc_data(SPACINGS, (prev_typ, typ))
            }?;

            Some(build_common::make_glue(Measurement::Mu(*spacings), &glue_options).into())
        },
        dummy_prev,
        None,
        Some(dummy_next),
        is_root,
    );

    todo!()
}

// We use prev insert as a bool because the wacky callback method that KaTeX uses won't work in
// Rust nicely. As well, it isn't needed for all the uses of the function anyway.
fn traverse_non_space_nodes<F: Fn(&mut HtmlNode, &mut HtmlNode) -> Option<HtmlNode>>(
    nodes: &mut Vec<HtmlNode>,
    cb: &F,
    mut prev_node: HtmlNode,
    mut prev_insert_after: Option<usize>,
    next: Option<HtmlNode>,
    is_root: bool,
) {
    // Temporarily append the right node, if it exists
    let next_some = next.is_some();
    if let Some(next) = next {
        nodes.push(next);
    }

    let mut i = 0;
    while i < nodes.len() {
        let node = &mut nodes[i];
        if let Some(partial_group) = get_partial_group_children_mut(node) {
            traverse_non_space_nodes(
                partial_group,
                cb,
                prev_node.clone(),
                prev_insert_after,
                None,
                is_root,
            );
            continue;
        }

        let non_space = !node.node().classes.iter().any(|x| x == "mspace");
        let new_prev_node = if non_space {
            Some(node.clone())
        } else if is_root && node.node().classes.iter().any(|x| x == "newline") {
            Some(make_empty_span(vec!["leftmost".to_string()]).into())
        } else {
            None
        };

        // Ignore explicit spaces (e.g., \;, \,) when determining what implicit spacing should go
        // between atoms of different classes
        if non_space {
            if let Some(result) = cb(node, &mut prev_node) {
                if let Some(index) = prev_insert_after {
                    nodes.insert(index + 1, result);
                } else {
                    // insert at front
                    nodes.insert(0, result);
                }
                i += 1;
            }
        }

        if let Some(new_prev_node) = new_prev_node {
            prev_node = new_prev_node;
        }

        prev_insert_after = Some(i);
    }

    if next_some {
        nodes.pop();
    }
}

fn get_partial_group_children(node: &HtmlNode) -> Option<&[HtmlNode]> {
    Some(match node {
        HtmlNode::DocumentFragment(frag) => &frag.children,
        HtmlNode::Anchor(anchor) => &anchor.children,
        HtmlNode::Span(span) if span.node.classes.iter().any(|x| x == "enclosing") => {
            &span.children
        }
        _ => return None,
    })
}
fn get_partial_group_children_mut(node: &mut HtmlNode) -> Option<&mut Vec<HtmlNode>> {
    Some(match node {
        HtmlNode::DocumentFragment(frag) => &mut frag.children,
        HtmlNode::Anchor(anchor) => &mut anchor.children,
        HtmlNode::Span(span) if span.node.classes.iter().any(|x| x == "enclosing") => {
            &mut span.children
        }
        _ => return None,
    })
}

pub(crate) fn build_group(
    group: Option<&ParseNode>,
    options: &Options,
    base_options: Option<&Options>,
) -> HtmlNode {
    let Some(group) = group else {
        return make_empty_span(ClassList::new()).into();
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

                return group_node.into();
            }
        }

        group_node
    } else {
        panic!("Got group of unknown type");
    }
}

/// Return the outermost node of a dom tree
fn get_outermost_node(node: &HtmlNode, side: Side) -> &HtmlNode {
    if let Some(children) = get_partial_group_children(node) {
        if !children.is_empty() {
            return match side {
                Side::Left => get_outermost_node(&children.first().unwrap(), Side::Left),
                Side::Right => get_outermost_node(&children.last().unwrap(), Side::Right),
            };
        }
    }

    node
}

fn get_type_of_dom_tree(node: Option<&HtmlNode>, side: Option<Side>) -> Option<DomType> {
    let node = node?;

    let node = if let Some(side) = side {
        get_outermost_node(node, side)
    } else {
        node
    };

    let first_class = node.node().classes.first()?;

    // This makes a lot of assumptions as to where the type of atom appears. We should do a better
    // job of enforcing this.
    DomType::from_str(first_class.as_str())
}

pub(crate) fn make_null_delimiter(options: &Options, classes: ClassList) -> Span<HtmlNode> {
    let classes = options
        .base_sizing_classes()
        .into_iter()
        .chain(["nulldelimiter".to_string()])
        .chain(classes)
        .collect::<ClassList>();
    make_span_s(classes, Vec::new())
}
