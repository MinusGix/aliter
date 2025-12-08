use std::borrow::Cow;
use std::sync::Arc;

use crate::parse_node::{NodeInfo, ParseNode, ParseNodeType, PhantomNode, HPhantomNode, VPhantomNode};

use super::{ord_argument, FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let phantom = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Phantom, 1).with_allowed_in_text(true),
        handler: Box::new(phantom_handler),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });
    fns.insert(Cow::Borrowed("\\phantom"), phantom);

    let hphantom = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::HPhantom, 1).with_allowed_in_text(true),
        handler: Box::new(hphantom_handler),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });
    fns.insert(Cow::Borrowed("\\hphantom"), hphantom);

    let vphantom = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::VPhantomNode, 1)
            .with_allowed_in_text(true),
        handler: Box::new(vphantom_handler),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });
    fns.insert(Cow::Borrowed("\\vphantom"), vphantom);
}

fn phantom_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    ParseNode::Phantom(PhantomNode {
        body: ord_argument(args[0].clone()),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}

fn hphantom_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    ParseNode::HPhantom(HPhantomNode {
        body: Box::new(args[0].clone()),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}

fn vphantom_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    ParseNode::VPhantom(VPhantomNode {
        body: Box::new(args[0].clone()),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}
