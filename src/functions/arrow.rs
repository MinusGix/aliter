use std::sync::Arc;

use crate::parse_node::{NodeInfo, ParseNode, ParseNodeType, XArrowNode};
use crate::parser::ParseError;

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
        // TODO:
        #[cfg(feature = "html")]
        html_builder: None,
        // TODO
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });

    fns.insert_for_all_str(ARROW_NAMES.iter().copied(), arrow);
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
