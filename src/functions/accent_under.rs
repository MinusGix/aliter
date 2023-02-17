use std::sync::Arc;

use crate::parse_node::{AccentUnderNode, NodeInfo, ParseNode, ParseNodeType};

use super::{FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let accent_under = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::AccentUnder, 1),
        handler: Box::new(accent_under_handler),
        // TODO:
        #[cfg(feature = "html")]
        html_builder: None,
        // TODO
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });

    fns.insert_for_all_str(ACCENT_UNDER_NAMES.iter().copied(), accent_under);
}

const ACCENT_UNDER_NAMES: &'static [&'static str] = &[
    "\\underleftarrow",
    "\\underrightarrow",
    "\\underleftrightarrow",
    "\\undergroup",
    "\\underlinesegment",
    "\\utilde",
];

fn accent_under_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    let base = args[0].clone();
    ParseNode::AccentUnder(AccentUnderNode {
        label: ctx.func_name.into_owned(),
        is_stretchy: None,
        is_shifty: None,
        base: Box::new(base),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}

// TODO: html/mathml
