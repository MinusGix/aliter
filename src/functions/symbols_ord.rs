use std::sync::Arc;

use crate::{
    build_common,
    expander::Mode,
    mathml,
    mathml_tree::{MathNode, MathNodeType},
    parse_node::{ParseNode, ParseNodeType},
    tree::ClassList,
    util::{find_assoc_data, FontVariant},
};

use super::{BuilderFunctionSpec, FunctionPropSpec, Functions};

const DEFAULT_VARIANT: &'static [(MathNodeType, FontVariant)] = &[
    (MathNodeType::Mi, FontVariant::Italic),
    (MathNodeType::Mn, FontVariant::Normal),
    (MathNodeType::MText, FontVariant::Normal),
];

pub fn add_functions(fns: &mut Functions) {
    let math_ord = Arc::new(BuilderFunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::MathOrd, 0),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            build_common::make_ord(group, options, build_common::OrdType::MathOrd)
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            let ParseNode::MathOrd(group) = group else {
                panic!()
            };

            let text = mathml::make_text(group.text.clone(), group.info.mode, Some(options));
            let mut node = MathNode::new(MathNodeType::Mi, vec![text], ClassList::new());

            let variant = mathml::get_variant(group, options).unwrap_or(FontVariant::Italic);

            if Some(&variant) != find_assoc_data(DEFAULT_VARIANT, node.typ) {
                node.set_attribute("mathvariant", variant.as_str());
            }

            node.into()
        })),
    });

    fns.insert_builder(math_ord);

    let text_ord = Arc::new(BuilderFunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::TextOrd, 0),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            build_common::make_ord(group, options, build_common::OrdType::TextOrd)
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            let ParseNode::TextOrd(group) = group else {
                panic!()
            };

            let text = mathml::make_text(group.text.to_string(), group.info.mode, Some(options));
            let variant = mathml::get_variant(group, options).unwrap_or(FontVariant::Normal);

            let node_type = if group.info.mode == Mode::Text {
                MathNodeType::MText
            } else if group
                .text
                .chars()
                .nth(0)
                .map(|c| c.is_digit(10))
                .unwrap_or(false)
            {
                MathNodeType::Mn
            } else if group.text == "\\prime" {
                MathNodeType::Mo
            } else {
                MathNodeType::Mi
            };

            let mut node = MathNode::new(node_type, vec![text], ClassList::new());

            if Some(&variant) != find_assoc_data(DEFAULT_VARIANT, node.typ) {
                node.set_attribute("mathvariant", variant.as_str());
            }

            node.into()
        })),
    });

    fns.insert_builder(text_ord);
}
