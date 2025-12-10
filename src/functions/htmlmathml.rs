use std::{borrow::Cow, sync::Arc};

use crate::{
    build_common, html, mathml,
    parse_node::{HtmlMathmlNode, NodeInfo, ParseNode, ParseNodeType},
    parser::ParseError,
};

use super::{ord_argument, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let h = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::HtmlMathml, 2)
            .with_allowed_in_text(true),
        handler: Box::new(|ctx, args, _opt_args| {
            Ok(ParseNode::HtmlMathml(HtmlMathmlNode {
                html: ord_argument(args[0].clone()),
                mathml: ord_argument(args[1].clone()),
                info: NodeInfo::new_mode(ctx.parser.mode()),
            }))
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            let ParseNode::HtmlMathml(group) = group else {
                panic!();
            };

            let elements =
                html::build_expression(&group.html, options, html::RealGroup::False, (None, None));

            build_common::make_fragment(elements).into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            let ParseNode::HtmlMathml(group) = group else {
                panic!();
            };
            mathml::build_expression_row(&group.mathml, options, None)
        })),
    });

    fns.insert(Cow::Borrowed("\\html@mathml"), h);
}
