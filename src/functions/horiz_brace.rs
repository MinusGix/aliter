use std::borrow::Cow;
use std::sync::Arc;

use crate::parse_node::{HorizBraceNode, NodeInfo, ParseNode, ParseNodeType};

use super::{FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let horiz_brace = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::HorizBrace, 1),
        handler: Box::new(horiz_brace_handler),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });

    fns.insert(Cow::Borrowed("\\overbrace"), horiz_brace.clone());
    fns.insert(Cow::Borrowed("\\underbrace"), horiz_brace);
}

fn horiz_brace_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    let func_name = ctx.func_name.into_owned();
    ParseNode::HorizBrace(HorizBraceNode {
        label: func_name.clone(),
        is_over: func_name.starts_with("\\over"),
        base: Box::new(args[0].clone()),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}
