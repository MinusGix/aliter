use std::sync::Arc;

use crate::{
    build_common::make_glue,
    mathml_tree::SpaceNode,
    parse_node::{KernNode, NodeInfo, ParseNode, ParseNodeType},
    unit::Em,
    util::ArgType,
};

use super::{FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let kern = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Kern, 1)
            .with_allowed_in_text(true)
            .with_arg_types(&[ArgType::Size] as &[ArgType]),
        handler: Box::new(|ctx, args, _opt_args| {
            let ParseNode::Size(size) = args[0].clone() else { panic!() };
            // TODO: strict wanrings

            ParseNode::Kern(KernNode {
                dimension: size.value,
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            let ParseNode::Kern(group) = group else { unreachable!() };
            make_glue(group.dimension.clone(), options).into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, _| {
            let ParseNode::Kern(group) = group else { unreachable!() };
            // TODO: KaTeX seems to assume that kern will only ever get Em in mathml?
            let dim = Em(group.dimension.num());
            SpaceNode::new(dim).into()
        })),
    });

    fns.insert_for_all_str(
        ["\\kern", "\\mkern", "\\hskip", "\\mskip"].into_iter(),
        kern,
    );
}
