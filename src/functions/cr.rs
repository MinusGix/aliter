use std::sync::Arc;

use crate::{
    parse_node::{CrNode, NodeInfo, ParseNode, ParseNodeType},
    util::ArgType,
};

use super::{FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let cr = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_opt_args(ParseNodeType::Cr, 0, 1)
            .with_allowed_in_text(true)
            .with_arg_types(&[ArgType::Size] as &[ArgType]),
        handler: Box::new(cr_handler),
        // TODO:
        #[cfg(feature = "html")]
        html_builder: None,
    });

    fns.insert("\\\\".into(), cr);
}

fn cr_handler(
    ctx: FunctionContext,
    _args: &[ParseNode],
    opt_args: &[Option<ParseNode>],
) -> ParseNode {
    let size = opt_args[0].as_ref().map(|arg| {
        if let ParseNode::Size(size) = arg {
            size.value.clone()
        } else {
            // TODO: don't panic
            panic!()
        }
    });
    // TODO: use strict behavior
    let new_line = !ctx.parser.conf.display_mode;

    ParseNode::Cr(CrNode {
        new_line,
        size,
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}
