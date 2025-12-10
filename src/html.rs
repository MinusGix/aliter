use std::borrow::Cow;

use crate::{
    build_common::{self, make_empty_span, make_span, make_span_s, make_v_list, VListElemShift, VListParam},
    dom_tree::{CssStyle, HtmlNode, Span, WithHtmlDomNode},
    functions,
    parse_node::{ArrayNode, ArrayTag, ParseNode},
    style::SCRIPT_STYLE,
    spacing_data::{SPACINGS, TIGHT_SPACINGS},
    tree::ClassList,
    unit::{calculate_size, make_em, Measurement},
    util::{find_assoc_data, has_class},
    Options,
};

// Binary atoms (first class `mbin`) change into ordinary atoms (`mord`)
// depending on their surroundings. See TeXbook pg. 442-446, Rules 5 and 6,
// and the text before Rule 19.
const BIN_LEFT_CANCELLER: &'static [&'static str] =
    &["leftmost", "mbin", "mopen", "mrel", "mop", "mpunct"];
const BIN_RIGHT_CANCELLER: &'static [&'static str] = &["rightmost", "mrel", "mclose", "mpunct"];

#[allow(dead_code)]
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

/// Take an entire parse tree and build it into an appropriate set of HTML nodes.
pub(crate) fn build_html(tree: &[ParseNode], options: &Options) -> Span<HtmlNode> {
    // Strip off any outer tag wrapper
    let (tag, tree) = if tree.len() == 1 && matches!(tree[0], ParseNode::Tag(_)) {
        let ParseNode::Tag(tag) = tree.iter().nth(0).unwrap() else {
            unreachable!()
        };

        (Some(&tag.tag), tag.body.as_slice())
    } else {
        (None, tree)
    };

    let mut expression = build_expression(&tree, &options, RealGroup::Root, (None, None));

    let eqn_num =
        if expression.len() == 2 && expression[1].node().classes.iter().any(|c| c == "tag") {
            // An environment with automatic equation numbers, e.g. {gather}.
            expression.pop()
        } else {
            None
        };

    let mut children: Vec<HtmlNode> = Vec::new();

    // Create one base node for each chunk between potential line breaks.
    // The TeXBook [p.173] says "A formula will be broken only after a
    // relation symbol like $=$ or $<$ or $\rightarrow$, or after a binary
    // operation symbol like $+$ or $-$ or $\times$, where the relation or
    // binary operation is on the ``outer level'' of the formula (i.e., not
    // enclosed in {...} and not part of an \over construction)."

    let mut parts = Vec::new();
    let mut expression_iter = expression.into_iter().peekable();
    while let Some(expr) = expression_iter.next() {
        // TODO: don't clone
        parts.push(expr.clone());

        let classes = &expr.node().classes;
        if has_class(classes, "mbin")
            || has_class(classes, "mrel")
            || has_class(classes, "allowbreak")
        {
            // Put any post-operator glue on the same line as the operator.
            // Watch for \nobreak along the way, and stop at \newline.
            let mut nobreak = false;
            while let Some(next_expr) = expression_iter.peek() {
                let next_classes = &next_expr.node().classes;
                if !(has_class(next_classes, "mspace") && !has_class(next_classes, "newline")) {
                    break;
                }

                let next_expr = expression_iter.next().unwrap();

                if has_class(&next_expr.node().classes, "nobreak") {
                    nobreak = true;
                }

                parts.push(next_expr);
            }

            // Don't allow break if \nobreak among the post-operator glue.
            if !nobreak {
                children.push(build_html_unbreakable(parts, &options).into());
                parts = Vec::new();
            }
        } else if has_class(classes, "newline") {
            // Write the line except the newline
            parts.pop();
            if !parts.is_empty() {
                children.push(build_html_unbreakable(parts, &options).into());
                parts = Vec::new();
            }

            // Put the newline at the top level
            children.push(expr);
        }
    }

    if !parts.is_empty() {
        children.push(build_html_unbreakable(parts, &options).into());
    }

    // Now, if there was a tag, build it too and append it as a final child.
    let has_tag_child = tag.is_some();
    if let Some(tag) = tag {
        let tag_child = build_expression(&tag, &options, RealGroup::True, (None, None));
        let mut tag_child = build_html_unbreakable(tag_child, &options);
        tag_child.node.classes = vec!["tag".to_string()];

        children.push(tag_child.into());
    } else if let Some(eqn_num) = eqn_num {
        children.push(eqn_num);
    }

    let mut html_node = make_span_s(vec!["katex-html".to_string()], children);
    html_node
        .attributes
        .insert("aria-hidden".to_string(), "true".to_string());

    // Adjust the strut of the tag to be the maximum height of all children
    // (the height of the enclosing htmlNode) for proper vertical alignment.
    if has_tag_child {
        let height = html_node.node().height;
        let depth = html_node.node().depth;
        let strut = html_node.children.last_mut().unwrap();
        strut.node_mut().style.height = Some(Cow::Owned(make_em(height + depth)));

        // TODO: katex includes it if it is defined. Do they use strings for this so it could be undefined? Should we do the same?
        // if depth != 0.0 {
        strut.node_mut().style.vertical_align = Some(Cow::Owned(make_em(-depth)));
        // }
    }

    html_node
}

/// Combine an array of HTML DOM nodes into an unbreakable HTML node of class `.base`, with proper
/// struts to guarantee correct vertical extent. [`build_html`] calls this repeatedly to make up
/// the entire expression as a sequence of unbreakable units.
fn build_html_unbreakable(children: Vec<HtmlNode>, options: &Options) -> Span<HtmlNode> {
    // Compute height and depth of this chunk.
    let mut body = make_span(
        vec!["base".to_string()],
        children,
        Some(options),
        CssStyle::default(),
    );

    // Add strut, which ensures that the top of the HTML element falls at the height of the
    // expression, and the bottom of the HTML element falls at the depth of the expression.
    let mut strut = make_empty_span(vec!["strut".to_string()]);
    strut.node.style.height = Some(Cow::Owned(make_em(body.node.height + body.node.depth)));
    // TODO: katex includes it if it is defined. Do they use strings for this so it could be undefined? Should we do the same?
    // if body.node.depth != 0.0 {
    strut.node.style.vertical_align = Some(Cow::Owned(make_em(-body.node.depth)));
    // }
    body.children.insert(0, strut.into());

    body
}

pub(crate) fn build_expression(
    expression: &[ParseNode],
    options: &Options,
    real_group: RealGroup,
    surrounding: (Option<DomType>, Option<DomType>),
) -> Vec<HtmlNode> {
    let mut groups = Vec::new();
    for expr in expression {
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

    groups
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
            i += 1;
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
        i += 1;
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
        panic!("Got group of unknown type: {:?}", group.typ());
    }
}

/// Build a simple HTML representation of an array/align-like group.
pub(crate) fn build_array(group: &ArrayNode, options: &Options) -> HtmlNode {
    let font_metrics = options.font_metrics();
    let rule_thickness = font_metrics
        .array_rule_width
        .max(options.min_rule_thickness.0);

    // Horizontal spacing defaults
    let pt = 1.0 / font_metrics.pt_per_em;
    let mut array_col_sep = 5.0 * pt;
    if matches!(
        group.col_separation_type,
        Some(crate::array::ColSeparationType::Small)
    ) {
        if let Some(local) = options.having_style(SCRIPT_STYLE) {
            array_col_sep = 0.2778 * (local.size_multiplier() / options.size_multiplier());
        } else {
            array_col_sep = 0.2778;
        }
    }

    // Vertical spacing defaults
    let base_line_skip = if matches!(
        group.col_separation_type,
        Some(crate::array::ColSeparationType::Cd)
    ) {
        calculate_size(&Measurement::Ex(crate::unit::Ex(3.0)), options)
    } else {
        12.0 * pt
    };
    let jot = 3.0 * pt;
    let array_skip = group.array_stretch * base_line_skip;
    let arstrut_height = 0.7 * array_skip;
    let arstrut_depth = 0.3 * array_skip;

    let mut total_height = 0.0f64;
    let mut hlines = Vec::new();
    let mut rows = Vec::new();

    let push_hlines =
        |height: &mut f64, target: &mut Vec<(f64, bool)>, lines_in_gap: &[bool]| {
            for (i, dashed) in lines_in_gap.iter().enumerate() {
                if i > 0 {
                    *height += 0.25;
                }
                target.push((*height, *dashed));
            }
        };

    push_hlines(&mut total_height, &mut hlines, &group.h_lines_before_row[0]);

    for (row_idx, row) in group.body.iter().enumerate() {
        let mut height = arstrut_height;
        let mut depth = arstrut_depth;

        let mut outrow: Vec<Option<HtmlNode>> = vec![None; row.len()];
        for (c, cell) in row.iter().enumerate() {
            let elt = build_group(Some(cell), options, None);
            height = height.max(elt.node().height);
            depth = depth.max(elt.node().depth);
            outrow[c] = Some(elt);
        }

        let mut gap = 0.0;
        if let Some(row_gap) = group.row_gaps.get(row_idx).and_then(|g| g.as_ref()) {
            gap = calculate_size(row_gap, options);
            if gap > 0.0 {
                gap += arstrut_depth;
                if depth < gap {
                    depth = gap;
                }
                gap = 0.0;
            }
        }
        if group.add_jot.unwrap_or(false) {
            depth += jot;
        }

        total_height += height;
        let pos = total_height;
        total_height += depth + gap;

        rows.push((outrow, height, depth, pos));

        if let Some(next) = group.h_lines_before_row.get(row_idx + 1) {
            push_hlines(&mut total_height, &mut hlines, next);
        }
    }

    let offset = total_height / 2.0 + font_metrics.axis_height;
    let mut nc = rows.iter().map(|r| r.0.len()).max().unwrap_or(0);

    // Build tag column if needed
    let mut tag_spans: Vec<VListElemShift<HtmlNode>> = Vec::new();
    if let Some(tags) = &group.tags {
        for (r, (_, height, depth, pos)) in rows.iter().enumerate() {
            let shift = pos - offset;
            if let Some(tag) = tags.get(r) {
                let mut tag_span: Span<HtmlNode> = match tag {
                    ArrayTag::Boolean(true) => build_common::make_span::<HtmlNode>(
                        vec!["eqn-num".to_string()],
                        Vec::new(),
                        Some(options),
                        CssStyle::default(),
                    ),
                    ArrayTag::Boolean(false) => make_empty_span(ClassList::new()).using_html_node(),
                    ArrayTag::Tag(body) => build_common::make_span::<HtmlNode>(
                        vec!["tag".to_string()],
                        build_expression(body, options, RealGroup::True, (None, None)),
                        Some(options),
                        CssStyle::default(),
                    ),
                };
                tag_span.node.height = *height;
                tag_span.node.depth = *depth;
                tag_spans.push(VListElemShift::new(tag_span.into(), shift));
            }
        }
    }

    let col_descriptions = group.cols.as_deref().unwrap_or(&[]);
    let mut cols: Vec<HtmlNode> = Vec::new();
    let hskip_before_and_after = group.h_skip_before_and_after.unwrap_or(false);

    // Build each column, interleaving separators and skips
    let mut col_descr_num = 0usize;
    let mut c = 0usize;
    while c < nc || col_descr_num < col_descriptions.len() {
        let mut col_descr = col_descriptions.get(col_descr_num);

        let mut first_separator = true;
        while let Some(crate::array::AlignSpec::Separator(sep)) = col_descr {
            if !first_separator {
                let mut sep_span = make_span::<HtmlNode>(
                    vec!["arraycolsep".to_string()],
                    Vec::new(),
                    Some(options),
                    CssStyle {
                        width: Some(Cow::Owned(make_em(font_metrics.double_rule_sep))),
                        ..CssStyle::default()
                    },
                );
                sep_span.node.depth = 0.0;
                sep_span.node.height = 0.0;
                cols.push(sep_span.into());
            }

            let mut separator = make_span::<HtmlNode>(
                vec!["vertical-separator".to_string()],
                Vec::new(),
                Some(options),
                CssStyle::default(),
            );
            separator.node.style.height = Some(Cow::Owned(make_em(total_height)));
            separator.node.style.border_right_width = Some(Cow::Owned(make_em(rule_thickness)));
            separator.node.style.border_right_style =
                Some(Cow::Owned(if sep.as_ref() == ":" { "dashed" } else { "solid" }.to_string()));
            separator.node.style.margin =
                Some(Cow::Owned(format!("0 {}", make_em(-rule_thickness / 2.0))));
            let shift = total_height - offset;
            if shift != 0.0 {
                separator
                    .node
                    .style
                    .vertical_align
                    .replace(Cow::Owned(make_em(-shift)));
            }
            cols.push(separator.into());

            col_descr_num += 1;
            col_descr = col_descriptions.get(col_descr_num);
            first_separator = false;
        }

        if c >= nc {
            col_descr_num += 1;
            continue;
        }

        let sepwidth_pre = if c > 0 || hskip_before_and_after {
            col_descr
                .and_then(|cd| {
                    if let crate::array::AlignSpec::Align { pre_gap, .. } = cd {
                        *pre_gap
                    } else {
                        None
                    }
                })
                .unwrap_or(array_col_sep)
        } else {
            0.0
        };
        if sepwidth_pre != 0.0 {
            let mut sep = make_span::<HtmlNode>(
                vec!["arraycolsep".to_string()],
                Vec::new(),
                Some(options),
                CssStyle {
                    width: Some(Cow::Owned(make_em(sepwidth_pre))),
                    ..CssStyle::default()
                },
            );
            sep.node.height = 0.0;
            sep.node.depth = 0.0;
            cols.push(sep.into());
        }

        let mut column_children: Vec<VListElemShift<HtmlNode>> = Vec::new();
        for (r_idx, (row, height, depth, pos)) in rows.iter().enumerate() {
            if let Some(elem) = row.get(c).and_then(|x| x.as_ref()) {
                let mut elem = elem.clone();
                elem.node_mut().height = *height;
                elem.node_mut().depth = *depth;
                column_children.push(VListElemShift::new(elem.clone(), pos - offset));
            } else if let Some(tags) = &group.tags {
                // Even if the cell is missing, tags may introduce a column count
                if tags.get(r_idx).is_some() && nc < c + 1 {
                    nc = c + 1;
                }
            }
        }

        if column_children.is_empty() {
            c += 1;
            col_descr_num += 1;
            continue;
        }

        let mut col_vlist = make_v_list(
            VListParam::IndividualShift {
                children: column_children,
            },
            options,
        );

        let align_class = if let Some(crate::array::AlignSpec::Align { align, .. }) = col_descr {
            format!("col-align-{}", align.as_ref())
        } else {
            "col-align-c".to_string()
        };
        col_vlist
            .node
            .classes
            .push(align_class);
        cols.push(col_vlist.into());

        let sepwidth_post = if c + 1 < nc || hskip_before_and_after {
            col_descr
                .and_then(|cd| {
                    if let crate::array::AlignSpec::Align { post_gap, .. } = cd {
                        *post_gap
                    } else {
                        None
                    }
                })
                .unwrap_or(array_col_sep)
        } else {
            0.0
        };
        if sepwidth_post != 0.0 {
            let mut sep = make_span::<HtmlNode>(
                vec!["arraycolsep".to_string()],
                Vec::new(),
                Some(options),
                CssStyle {
                    width: Some(Cow::Owned(make_em(sepwidth_post))),
                    ..CssStyle::default()
                },
            );
            sep.node.height = 0.0;
            sep.node.depth = 0.0;
            cols.push(sep.into());
        }

        c += 1;
        col_descr_num += 1;
    }

    let mut body = make_span::<HtmlNode>(vec!["mtable".to_string()], cols, Some(options), CssStyle::default());

    if !hlines.is_empty() {
        let mut vlist_children: Vec<VListElemShift<HtmlNode>> = Vec::new();
        vlist_children.push(VListElemShift::new(body.clone().into(), 0.0));

        for (pos, dashed) in hlines.into_iter().rev() {
            let class = if dashed { "hdashline" } else { "hline" };
            let line = build_common::make_line_span(class, options, Some(rule_thickness));
            vlist_children.push(VListElemShift::new(line.into(), pos - offset));
        }

        body = make_v_list(VListParam::IndividualShift { children: vlist_children }, options);
    }

    let final_node: HtmlNode = if tag_spans.is_empty() {
        build_common::make_span::<HtmlNode>(
            vec!["mord".to_string()],
            vec![body.into()],
            Some(options),
            CssStyle::default(),
        )
        .into()
    } else {
        let mut eqn_num_col = make_v_list(
            VListParam::IndividualShift {
                children: tag_spans,
            },
            options,
        );
        eqn_num_col.node.classes.push("tag".to_string());
        build_common::make_fragment::<HtmlNode>(vec![body.into(), eqn_num_col.into()]).into()
    };

    final_node
}

/// Build HTML for a left-right delimiter group
pub(crate) fn build_leftright(group: &crate::parse_node::LeftRightNode, options: &Options) -> HtmlNode {
    // Build the inner expression
    let mut inner = build_expression(
        &group.body,
        options,
        RealGroup::True,
        (Some(crate::html::DomType::MOpen), Some(crate::html::DomType::MClose)),
    );

    let mut inner_height = 0.0f64;
    let mut inner_depth = 0.0f64;

    // Calculate height and depth of inner content
    for node in &inner {
        inner_height = inner_height.max(node.node().height);
        inner_depth = inner_depth.max(node.node().depth);
    }

    // Scale by size multiplier
    inner_height *= options.size_multiplier();
    inner_depth *= options.size_multiplier();

    // Build left delimiter
    let left_delim = if group.left == "." {
        make_null_delimiter(options, vec!["mopen".to_string()]).into()
    } else {
        crate::delimiter::left_right_delim(
            &group.left,
            inner_height,
            inner_depth,
            options,
            group.info.mode,
            vec!["mopen".to_string()],
        ).into()
    };
    inner.insert(0, left_delim);

    // Build right delimiter
    let right_delim = if group.right == "." {
        make_null_delimiter(options, vec!["mclose".to_string()]).into()
    } else {
        let right_options = if let Some(ref color) = group.right_color {
            options.clone_alter().with_color(color.clone())
        } else {
            options.clone()
        };
        crate::delimiter::left_right_delim(
            &group.right,
            inner_height,
            inner_depth,
            &right_options,
            group.info.mode,
            vec!["mclose".to_string()],
        ).into()
    };
    inner.push(right_delim);

    build_common::make_span(vec!["minner".to_string()], inner, Some(options), CssStyle::default()).into()
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
