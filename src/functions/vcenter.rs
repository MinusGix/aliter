use std::borrow::Cow;
use std::sync::Arc;

use crate::{
    build_common,
    dom_tree::WithHtmlDomNode,
    html, mathml,
    mathml_tree::{MathNode, MathNodeType},
    parse_node::{NodeInfo, ParseNode, ParseNodeType, VCenterNode},
    tree::ClassList,
};

use super::{FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let vcenter = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::VCenter, 1)
            .with_allowed_in_text(false)
            .with_arg_types(&[crate::util::ArgType::Original] as &[crate::util::ArgType]),
        handler: Box::new(|ctx, args, _| {
            ParseNode::VCenter(VCenterNode {
                body: Box::new(args[0].clone()),
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            let ParseNode::VCenter(group) = group else { unreachable!() };
            let body = html::build_group(Some(&group.body), options, None);
            let axis_height = options.font_metrics().axis_height;
            let dy = 0.5 * ((body.node().height - axis_height) - (body.node().depth + axis_height));
            build_common::make_v_list(
                build_common::VListParam::Shift {
                    amount: dy,
                    children: vec![build_common::VListShiftChild::Elem(
                        build_common::VListElem::new(body),
                    )],
                },
                options,
            )
            .into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            let ParseNode::VCenter(group) = group else { unreachable!() };
            let mut node = MathNode::new(
                MathNodeType::MPadded,
                vec![mathml::build_group(Some(&group.body), options)],
                ClassList::new(),
            );
            node.classes.push("vcenter".to_string());
            node.into()
        })),
    });

    fns.insert(Cow::Borrowed("\\vcenter"), vcenter);
}
