use std::sync::Arc;

use crate::parse_node::{NodeInfo, ParseNode, ParseNodeType, XArrowNode};
use crate::parser::ParseError;

#[cfg(feature = "html")]
use crate::{
    build_common::{make_span, make_v_list, VListElemShift, VListParam},
    dom_tree::{CssStyle, HtmlNode, WithHtmlDomNode},
    html,
    stretchy,
    Options,
};

#[cfg(feature = "mathml")]
use crate::{
    mathml,
    mathml_tree::{MathNode, MathNodeType, MathmlNode},
    stretchy as stretchy_mathml,
    tree::ClassList,
};

use super::{FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

const ARROW_NAMES: &'static [&'static str] = &[
    "\\xleftarrow",
    "\\xrightarrow",
    "\\xLeftarrow",
    "\\xRightarrow",
    "\\xleftrightarrow",
    "\\xLeftrightarrow",
    "\\xhookleftarrow",
    "\\xhookrightarrow",
    "\\xmapsto",
    "\\xrightharpoondown",
    "\\xrightharpoonup",
    "\\xleftharpoondown",
    "\\xleftharpoonup",
    "\\xrightleftharpoons",
    "\\xleftrightharpoons",
    "\\xlongequal",
    "\\xtwoheadrightarrow",
    "\\xtwoheadleftarrow",
    "\\xtofrom",
    // The next 3 functions are here to support the mhchem extension.
    // Direct use of these functions is discouraged and may break someday.
    "\\xrightleftarrows",
    "\\xrightequilibrium",
    "\\xleftequilibrium",
    // The next 3 functions are here only to support the {CD} environment.
    "\\\\cdrightarrow",
    "\\\\cdleftarrow",
    "\\\\cdlongequal",
];

pub fn add_functions(fns: &mut Functions) {
    let arrow = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_opt_args(ParseNodeType::XArrow, 1, 1),
        handler: Box::new(arrow_handler),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });

    fns.insert_for_all_str(ARROW_NAMES.iter().copied(), arrow);
}

#[cfg(feature = "html")]
fn html_builder(group: &ParseNode, options: &Options) -> HtmlNode {
    let ParseNode::XArrow(group) = group else {
        panic!("Expected XArrow node");
    };

    let style = &options.style;

    // Build the argument groups in the appropriate style
    // Ref: amsmath.dtx: \hbox{$\scriptstyle\mkern#3mu{#6}\mkern#4mu$}%
    let new_options = options.having_style(style.sup()).unwrap_or_else(|| options.clone());
    let upper_group = html::build_group(Some(&group.body), &new_options, None);

    // Determine arrow prefix for class naming
    let arrow_prefix = if group.label.starts_with("\\x") { "x" } else { "cd" };

    // Add class to upper group
    let upper_span = make_span::<HtmlNode>(
        vec![format!("{}-arrow-pad", arrow_prefix)],
        vec![upper_group],
        Some(&new_options),
        CssStyle::default(),
    );

    // Build lower group if present
    let lower_group = if let Some(below) = &group.below {
        let new_options_sub = options.having_style(style.sub()).unwrap_or_else(|| options.clone());
        let lower = html::build_group(Some(below), &new_options_sub, None);
        Some(make_span::<HtmlNode>(
            vec![format!("{}-arrow-pad", arrow_prefix)],
            vec![lower],
            Some(&new_options_sub),
            CssStyle::default(),
        ))
    } else {
        None
    };

    // Create the arrow SVG span
    let arrow_body = stretchy::svg_span(&group.label, options);

    // Get font metrics for positioning
    let axis_height = options.font_metrics().axis_height;
    let arrow_height = arrow_body.node().height;

    // Calculate shifts
    let arrow_shift = -axis_height + 0.5 * arrow_height;
    let mut upper_shift = -axis_height - 0.5 * arrow_height - 0.111; // 0.111 em = 2 mu

    if upper_span.node().depth > 0.25 || group.label == "\\xleftequilibrium" {
        upper_shift -= upper_span.node().depth;
    }

    // Generate the vlist
    let vlist = if let Some(lower) = lower_group {
        let lower_shift = -axis_height + lower.node().height + 0.5 * arrow_height + 0.111;

        make_v_list(
            VListParam::IndividualShift {
                children: vec![
                    VListElemShift::new(upper_span.into(), upper_shift),
                    VListElemShift::new(arrow_body, arrow_shift),
                    VListElemShift::new(lower.into(), lower_shift),
                ],
            },
            options,
        )
    } else {
        make_v_list(
            VListParam::IndividualShift {
                children: vec![
                    VListElemShift::new(upper_span.into(), upper_shift),
                    VListElemShift::new(arrow_body, arrow_shift),
                ],
            },
            options,
        )
    };

    make_span::<HtmlNode>(
        vec!["mrel".to_string(), "x-arrow".to_string()],
        vec![vlist.into()],
        Some(options),
        CssStyle::default(),
    ).into()
}

/// Helper function to create a padded node for MathML
#[cfg(feature = "mathml")]
fn padded_node(group: Option<MathmlNode>) -> MathmlNode {
    let children: Vec<MathmlNode> = group.map(|g| vec![g]).unwrap_or_default();
    let mut node: MathNode<MathmlNode> = MathNode::new(MathNodeType::MPadded, children, ClassList::new());
    node.set_attribute("width", "+0.6em");
    node.set_attribute("lspace", "0.3em");
    node.into()
}

#[cfg(feature = "mathml")]
fn mathml_builder(group: &ParseNode, options: &crate::Options) -> MathmlNode {
    let ParseNode::XArrow(group) = group else {
        panic!("Expected XArrow node");
    };

    let mut arrow_node = stretchy_mathml::mathml_node(&group.label);
    let min_size = if group.label.starts_with("\\x") { "1.75em" } else { "3.0em" };
    arrow_node.set_attribute("minsize", min_size);
    let arrow_mathml: MathmlNode = arrow_node.into();

    let upper_node = padded_node(Some(mathml::build_group(Some(&group.body), options)));

    let node: MathNode<MathmlNode> = if let Some(below) = &group.below {
        let lower_node = padded_node(Some(mathml::build_group(Some(below), options)));
        MathNode::new(
            MathNodeType::MUnderOver,
            vec![arrow_mathml, lower_node, upper_node],
            ClassList::new(),
        )
    } else {
        MathNode::new(
            MathNodeType::MOver,
            vec![arrow_mathml, upper_node],
            ClassList::new(),
        )
    };

    node.into()
}

fn arrow_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let body = Box::new(args[0].clone());
    let below = opt_args.get(0).cloned().flatten().map(Box::new);
    Ok(ParseNode::XArrow(XArrowNode {
        label: ctx.func_name.into_owned(),
        body,
        below,
        info: NodeInfo::new_mode(ctx.parser.mode()),
    }))
}
