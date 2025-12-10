use std::sync::Arc;

use crate::{
    parse_node::{InternalNode, NodeInfo, ParseNode, ParseNodeType},
    parser::ParseError,
};

use super::{FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let relax = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Internal, 0)
            .with_allowed_in_text(true),
        handler: Box::new(|ctx, _args, _opt_args| {
            Ok(ParseNode::Internal(InternalNode {
                info: NodeInfo::new_mode(ctx.parser.mode()),
            }))
        }),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });

    fns.insert("\\relax".into(), relax);
}
