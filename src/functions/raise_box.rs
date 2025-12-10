use std::{borrow::Cow, sync::Arc};

use crate::{
    build_common::{self, VListElem, VListParam, VListShiftChild},
    html, mathml,
    mathml_tree::{MathNode, MathNodeType},
    parse_node::{NodeInfo, ParseNode, ParseNodeType, RaiseBoxNode},
    parser::ParseError,
    tree::ClassList,
    unit::calculate_size,
    util::ArgType,
};

use super::{FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let raise = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::RaiseBox, 2)
            .with_allowed_in_text(true)
            .with_arg_types(&[ArgType::Size, ArgType::HBox] as &[ArgType]),
        handler: Box::new(|ctx, args, _opt_args| {
            let ParseNode::Size(size) = args[0].clone() else { panic!() };
            let size = size.value;
            let body = args[1].clone();

            Ok(ParseNode::RaiseBox(RaiseBoxNode {
                dy: size,
                body: Box::new(body),
                info: NodeInfo::new_mode(ctx.parser.mode()),
            }))
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            let ParseNode::RaiseBox(group) = group else { unreachable!() };
            let body = html::build_group(Some(&group.body), options, None);
            let dy = calculate_size(&group.dy, options);
            build_common::make_v_list(
                VListParam::Shift {
                    amount: -dy,
                    children: vec![VListShiftChild::Elem(VListElem::new(body))],
                },
                options,
            )
            .into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            let ParseNode::RaiseBox(group) = group else { unreachable!() };
            let node = mathml::build_group(Some(&group.body), options);
            let mut node = MathNode::new(MathNodeType::MPadded, vec![node], ClassList::new());
            let dy = group.dy.to_string();
            node.set_attribute("voffset", dy);

            node.into()
        })),
    });

    fns.insert(Cow::Borrowed("\\raisebox"), raise);
}
