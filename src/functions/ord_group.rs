use std::sync::Arc;

use crate::{
    build_common::{make_fragment, make_span},
    dom_tree::CssStyle,
    html::{self, RealGroup},
    mathml,
    parse_node::{ParseNode, ParseNodeType},
};

use super::{BuilderFunctionSpec, FunctionPropSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let ord = Arc::new(BuilderFunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::OrdGroup, 0),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            let ParseNode::OrdGroup(group) = group else {
                panic!();
            };

            let expr = html::build_expression(&group.body, options, RealGroup::False, (None, None));

            if group.semi_simple == Some(true) {
                make_fragment(expr).into()
            } else {
                make_span(
                    vec!["mord".to_string()],
                    expr,
                    Some(options),
                    CssStyle::default(),
                )
                .into()
            }
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            let ParseNode::OrdGroup(group) = group else {
                panic!();
            };

            mathml::build_expression_row(&group.body, options, Some(true))
        })),
    });

    fns.insert_builder(ord);
}
