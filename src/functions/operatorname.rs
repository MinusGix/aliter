use std::sync::Arc;

use crate::parse_node::{NodeInfo, OperatorNameNode, ParseNode, ParseNodeType};

use super::{ord_argument, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let op = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::OperatorName, 1),
        handler: Box::new(|ctx, args, _| {
            ParseNode::OperatorName(OperatorNameNode {
                body: ord_argument(args[0].clone()),
                always_handle_sup_sub: false,
                limits: false,
                parent_is_sup_sub: false,
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });
    fns.insert("\\operatorname@".into(), op.clone());

    let op_limits = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::OperatorName, 1),
        handler: Box::new(|ctx, args, _| {
            ParseNode::OperatorName(OperatorNameNode {
                body: ord_argument(args[0].clone()),
                always_handle_sup_sub: true,
                limits: false,
                parent_is_sup_sub: false,
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });
    fns.insert("\\operatornamewithlimits".into(), op_limits);
}
