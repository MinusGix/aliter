use std::borrow::Cow;
use std::sync::Arc;

use crate::parse_node::{HBoxNode, NodeInfo, ParseNode, ParseNodeType};
use crate::util::ArgType;

use super::{FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let hbox = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::HBox, 1)
            .with_allowed_in_text(true)
            .with_primitive(true)
            .with_arg_types(&[ArgType::Raw] as &[ArgType]),
        handler: Box::new(|ctx, _args, _| {
            // We ignore the raw content for now; \hbox acts as a container barrier.
            ParseNode::HBox(HBoxNode {
                body: Vec::new(),
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });

    fns.insert(Cow::Borrowed("\\hbox"), hbox);
}
