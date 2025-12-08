use std::borrow::Cow;
use std::sync::Arc;

use crate::{
    parse_node::{IncludeGraphicsNode, NodeInfo, ParseNode, ParseNodeType},
    unit::{Em, Measurement},
    util::ArgType,
};

use super::{FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let includegraphics = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_opt_args(ParseNodeType::IncludeGraphics, 1, 1)
            .with_allowed_in_text(true)
            .with_arg_types(&[ArgType::Raw, ArgType::Url] as &[ArgType]),
        handler: Box::new(includegraphics_handler),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });

    fns.insert(Cow::Borrowed("\\includegraphics"), includegraphics);
}

fn includegraphics_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    opt_args: &[Option<ParseNode>],
) -> ParseNode {
    let _attrs = opt_args.get(0);

    let src = if let ParseNode::Url(url) = &args[0] {
        url.url.clone()
    } else {
        String::new()
    };

    ParseNode::IncludeGraphics(IncludeGraphicsNode {
        alt: String::new(),
        width: Measurement::Em(Em(0.0)),
        height: Measurement::Em(Em(0.9)),
        total_height: Measurement::Em(Em(0.0)),
        src,
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}
