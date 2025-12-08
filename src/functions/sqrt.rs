use std::sync::Arc;

use crate::{
    mathml,
    parse_node::{NodeInfo, ParseNode, ParseNodeType, SqrtNode},
};

use super::{FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

#[cfg(feature = "mathml")]
use crate::mathml_tree::{MathNode, MathNodeType, MathmlNode};
#[cfg(feature = "mathml")]
use crate::tree::ClassList;

pub fn add_functions(fns: &mut Functions) {
    let sqrt = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_opt_args(ParseNodeType::Sqrt, 1, 1)
            .with_allowed_in_argument(true),
        handler: Box::new(sqrt_handler),
        #[cfg(feature = "html")]
        html_builder: None, // TODO: Implement html builder
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });
    fns.insert("\\sqrt".into(), sqrt);
}

fn sqrt_handler(
    ctx: FunctionContext,
    args: & [ParseNode],
    opt_args: &[Option<ParseNode>],
) -> ParseNode {
    let index = opt_args[0].clone().map(Box::new);
    let body = Box::new(args[0].clone());
    ParseNode::Sqrt(SqrtNode {
        body,
        index,
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}

#[cfg(feature = "mathml")]
fn mathml_builder(group: &ParseNode, options: &crate::Options) -> MathmlNode {
    let ParseNode::Sqrt(node) = group else {
        panic!("Expected Sqrt node");
    };
    let body = mathml::build_group(Some(&node.body), options);
    if let Some(index) = &node.index {
        let index = mathml::build_group(Some(index), options);
        MathNode::new(MathNodeType::MRoot, vec![body, index], ClassList::new()).into()
    } else {
        MathNode::new(MathNodeType::MSqrt, vec![body], ClassList::new()).into()
    }
}