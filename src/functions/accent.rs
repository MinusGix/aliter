use std::sync::Arc;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    expander::Mode,
    parse_node::{AccentNode, NodeInfo, ParseNode, ParseNodeType},
    util::ArgType,
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
) -> ParseNode {
    let base = normalize_argument(args[0].clone());

    let is_stretchy = !NON_STRETCHY_ACCENT_REGEX.is_match(&ctx.func_name);
    let is_shifty = !is_stretchy
        || ctx.func_name == "\\widehat"
        || ctx.func_name == "\\widetilde"
        || ctx.func_name == "\\widecheck";

    ParseNode::Accent(AccentNode {
        label: ctx.func_name.into_owned().into(),
        is_stretchy: Some(is_stretchy),
        is_shifty: Some(is_shifty),
        base: Box::new(base),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
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
    });

    fns.insert_for_all_str(TEXT_MODE_ACCENT_NAMES.iter().copied(), accent);
}

fn text_mode_accent_handler<'a, 'p, 'i, 'f>(
    ctx: FunctionContext<'a, 'p, 'i, 'f>,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    let base = &args[0];
    let mode = ctx.parser.mode();

    if mode == Mode::Math {
        // TODO: report non strict about the mode
    }

    ParseNode::Accent(AccentNode {
        label: ctx.func_name.into_owned().into(),
        is_stretchy: Some(false),
        is_shifty: Some(true),
        base: Box::new(base.clone()),
        info: NodeInfo::new_mode(Mode::Text),
    })
}

// TODO: HTML and MathML builders!
// behind features of course
