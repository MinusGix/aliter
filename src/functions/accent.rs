use std::sync::Arc;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    expander::Mode,
    parse_node::{AccentNode, NodeInfo, ParseNode, ParseNodeType},
    parser::ParseError,
    util::ArgType,
    dom_tree::{HtmlNode, WithHtmlDomNode}, // Added this line
};

use super::{normalize_argument, FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    add_accents(fns);
    add_text_mode_accents(fns);
}

const ACCENT_NAMES: &'static [&'static str] = &[
    "\\acute",
    "\\grave",
    "\\ddot",
    "\\tilde",
    "\\bar",
    "\\breve",
    "\\check",
    "\\hat",
    "\\vec",
    "\\dot",
    "\\mathring",
    "\\widecheck",
    "\\widehat",
    "\\widetilde",
    "\\overrightarrow",
    "\\overleftarrow",
    "\\Overrightarrow",
    "\\overleftrightarrow",
    "\\overgroup",
    "\\overlinesegment",
    "\\overleftharpoon",
    "\\overrightharpoon",
];

fn add_accents(fns: &mut Functions) {
    let accent = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Accent, 1),
        handler: Box::new(accent_handler),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            use crate::{build_common, html, stretchy};

            let ParseNode::Accent(group) = group else {
                panic!();
            };

            let base_node = html::build_group(Some(&group.base), options, None);

            let accent_node: HtmlNode = if group.is_stretchy.unwrap_or(false) {
                stretchy::svg_span(&group.label, options)
            } else {
                let accent_char = if group.label == "\\vec" {
                    "\u{2192}"
                } else {
                    &group.label
                };

                build_common::make_symbol(accent_char, "Main-Regular", group.info.mode, Some(options), Vec::new()).into()
            };

            let _accent_node_height = accent_node.node().height;
            let _accent_node_depth = accent_node.node().depth;

            let accent_shift = base_node.node().height;

            let mut vlist_children = Vec::new();
            vlist_children.push(build_common::VListElemShift::new(base_node, 0.0));
            vlist_children.push(build_common::VListElemShift::new(accent_node, accent_shift));

            let mut vlist = build_common::make_v_list(
                build_common::VListParam::IndividualShift {
                    children: vlist_children,
                },
                options,
            );

            if group.is_shifty.unwrap_or(false) {
                vlist.node.classes.push("accent-shifty".to_string());
            }

            vlist.into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(accent_mathml_builder)),
    });

    fns.insert_for_all_str(ACCENT_NAMES.iter().copied(), accent);
}

static NON_STRETCHY_ACCENT_REGEX: Lazy<Regex> = Lazy::new(|| {
    const REGEX_TEXT: &str =
    "\\\\acute|\\\\grave|\\\\ddot|\\\\tilde|\\\\bar|\\\\breve|\\\\check|\\\\hat|\\\\vec|\\\\dot|\\\\mathring";

    Regex::new(REGEX_TEXT).unwrap()
});

fn accent_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let base = normalize_argument(args[0].clone());

    let is_stretchy = !NON_STRETCHY_ACCENT_REGEX.is_match(&ctx.func_name);
    let is_shifty = !is_stretchy
        || ctx.func_name == "\\widehat"
        || ctx.func_name == "\\widetilde"
        || ctx.func_name == "\\widecheck";

    Ok(ParseNode::Accent(AccentNode {
        label: ctx.func_name.into_owned().into(),
        is_stretchy: Some(is_stretchy),
        is_shifty: Some(is_shifty),
        base: Box::new(base),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    }))
}

const TEXT_MODE_ACCENT_NAMES: &'static [&'static str] = &[
    "\\'",
    "\\`",
    "\\^",
    "\\~",
    "\\=",
    "\\u",
    "\\.",
    "\\\"",
    "\\c",
    "\\r",
    "\\H",
    "\\v",
    "\\textcircled",
];

// TODO: We could make so FunctionSpec has a function generic
// then since they're all arc'd we can make it dyn?
fn add_text_mode_accents(fns: &mut Functions) {
    let accent = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Accent, 1)
            .with_allowed_in_text(true)
            .with_allowed_in_math(true)
            .with_arg_types(&[ArgType::Primitive] as &[ArgType]),
        handler: Box::new(text_mode_accent_handler),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            use crate::{build_common, html, stretchy};

            let ParseNode::Accent(group) = group else {
                panic!();
            };

            let base_node = html::build_group(Some(&group.base), options, None);

            let accent_node: HtmlNode = if group.is_stretchy.unwrap_or(false) {
                stretchy::svg_span(&group.label, options)
            } else {
                let accent_char = if group.label == "\\vec" {
                    "\u{2192}"
                } else {
                    &group.label
                };

                build_common::make_symbol(accent_char, "Main-Regular", group.info.mode, Some(options), Vec::new()).into()
            };

            let _accent_node_height = accent_node.node().height;
            let _accent_node_depth = accent_node.node().depth;

            // Shift the accent node vertically. The exact positioning logic should be
            // derived from KaTeX's original implementation, but for a start, we'll
            // place it above the base.
            let accent_shift = base_node.node().height; // Simple start: place accent just above base

            let mut vlist_children = Vec::new();
            vlist_children.push(build_common::VListElemShift::new(base_node, 0.0));
            vlist_children.push(build_common::VListElemShift::new(accent_node, accent_shift));

            let mut vlist = build_common::make_v_list(
                build_common::VListParam::IndividualShift {
                    children: vlist_children,
                },
                options,
            );

            // Add classes for styling, e.g., for shifty accents.
            if group.is_shifty.unwrap_or(false) {
                vlist.node.classes.push("accent-shifty".to_string());
            }

            vlist.into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(accent_mathml_builder)),
    });

    fns.insert_for_all_str(TEXT_MODE_ACCENT_NAMES.iter().copied(), accent);
}

#[cfg(feature = "mathml")]
fn accent_mathml_builder(group: &ParseNode, options: &crate::Options) -> crate::mathml_tree::MathmlNode {
    use crate::{mathml, mathml_tree::{MathNode, MathNodeType, MathmlNode, TextNode}, stretchy, tree::ClassList};

    let ParseNode::Accent(group) = group else {
        panic!("Expected Accent node");
    };

    let accent_node: MathmlNode = if group.is_stretchy.unwrap_or(false) {
        stretchy::mathml_node(&group.label).into()
    } else {
        let text = mathml::make_text(group.label.to_string(), group.info.mode, Some(options));
        let mo: MathNode<MathmlNode> = MathNode::new(MathNodeType::Mo, vec![text.into()], ClassList::new());
        mo.into()
    };

    let base_node = mathml::build_group(Some(&group.base), options);

    let mut node: MathNode<MathmlNode> = MathNode::new(
        MathNodeType::MOver,
        vec![base_node, accent_node],
        ClassList::new(),
    );
    node.set_attribute("accent", "true");

    node.into()
}

fn text_mode_accent_handler<'a, 'p, 'i, 'f>(
    ctx: FunctionContext<'a, 'p, 'i, 'f>,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let base = &args[0];
    let mode = ctx.parser.mode();

    if mode == Mode::Math {
        // TODO: report non strict about the mode
    }

    Ok(ParseNode::Accent(AccentNode {
        label: ctx.func_name.into_owned().into(),
        is_stretchy: Some(false),
        is_shifty: Some(true),
        base: Box::new(base.clone()),
        info: NodeInfo::new_mode(Mode::Text),
    }))
}


