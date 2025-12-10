use std::borrow::Cow;
use std::sync::Arc;

use crate::parse_node::{NodeInfo, ParseNode, ParseNodeType, PhantomNode, HPhantomNode, VPhantomNode};

use super::{ord_argument, FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

#[cfg(any(feature = "html", feature = "mathml"))]
use crate::build_common;
#[cfg(feature = "html")]
use crate::{dom_tree::{CssStyle, WithHtmlDomNode}, html};
#[cfg(feature = "mathml")]
use crate::{mathml, mathml_tree::{MathNode, MathNodeType}};
#[cfg(feature = "html")]
use crate::tree::ClassList;

pub fn add_functions(fns: &mut Functions) {
    let phantom = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Phantom, 1).with_allowed_in_text(true),
        handler: Box::new(phantom_handler),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            let ParseNode::Phantom(phantom) = group else { panic!() };
            let elements = html::build_expression(
                &phantom.body,
                &options.clone_alter().with_phantom(),
                html::RealGroup::True,
                (None, None),
            );
            build_common::make_fragment(elements).into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            let ParseNode::Phantom(phantom) = group else { panic!() };
            let inner = mathml::build_expression(&phantom.body, options, None);
            MathNode::new(MathNodeType::MPhantom, inner, ClassList::new()).into()
        })),
    });
    fns.insert(Cow::Borrowed("\\phantom"), phantom);

    let hphantom = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::HPhantom, 1).with_allowed_in_text(true),
        handler: Box::new(hphantom_handler),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            let ParseNode::HPhantom(hphantom) = group else { panic!() };
            let mut node = build_common::make_span(
                Vec::new(),
                vec![html::build_group(Some(&hphantom.body), &options.clone_alter().with_phantom(), None)],
                Some(options),
                CssStyle::default(),
            );
            node.node.height = 0.0;
            node.node.depth = 0.0;

            // Zero out heights/depths of children
            for child in &mut node.children {
                child.node_mut().height = 0.0;
                child.node_mut().depth = 0.0;
            }

            // Wrap in vlist for baseline positioning
            let vlist_node = build_common::make_v_list(
                build_common::VListParam::FirstBaseLine {
                    children: vec![build_common::VListShiftChild::Elem(build_common::VListElem::new(node))],
                },
                options,
            );

            // For spacing, TeX treats as a math group (same spacing as ord)
            build_common::make_span::<crate::dom_tree::HtmlNode>(
                vec!["mord".to_string()],
                vec![vlist_node.into()],
                Some(options),
                CssStyle::default(),
            ).into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            let ParseNode::HPhantom(hphantom) = group else { panic!() };
            let inner = mathml::build_expression(&ord_argument(*hphantom.body.clone()), options, None);
            let phantom = MathNode::<crate::mathml_tree::MathmlNode>::new(MathNodeType::MPhantom, inner, ClassList::new());
            let mut node = MathNode::<crate::mathml_tree::MathmlNode>::new(MathNodeType::MPadded, vec![phantom.into()], ClassList::new());
            node.set_attribute("height", "0px");
            node.set_attribute("depth", "0px");
            node.into()
        })),
    });
    fns.insert(Cow::Borrowed("\\hphantom"), hphantom);

    let vphantom = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::VPhantomNode, 1)
            .with_allowed_in_text(true),
        handler: Box::new(vphantom_handler),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            let ParseNode::VPhantom(vphantom) = group else { panic!() };
            let inner = build_common::make_span(
                vec!["inner".to_string()],
                vec![html::build_group(Some(&vphantom.body), &options.clone_alter().with_phantom(), None)],
                Some(options),
                CssStyle::default(),
            );
            let fix = build_common::make_span::<crate::dom_tree::HtmlNode>(
                vec!["fix".to_string()],
                Vec::new(),
                Some(options),
                CssStyle::default(),
            );
            build_common::make_span::<crate::dom_tree::HtmlNode>(
                vec!["mord".to_string(), "rlap".to_string()],
                vec![inner.into(), fix.into()],
                Some(options),
                CssStyle::default(),
            ).into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            let ParseNode::VPhantom(vphantom) = group else { panic!() };
            let inner = mathml::build_expression(&ord_argument(*vphantom.body.clone()), options, None);
            let phantom = MathNode::<crate::mathml_tree::MathmlNode>::new(MathNodeType::MPhantom, inner, ClassList::new());
            let mut node = MathNode::<crate::mathml_tree::MathmlNode>::new(MathNodeType::MPadded, vec![phantom.into()], ClassList::new());
            node.set_attribute("width", "0px");
            node.into()
        })),
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
