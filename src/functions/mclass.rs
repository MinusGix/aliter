use std::{borrow::Cow, sync::Arc};

use crate::{
    build_common::make_span,
    html,
    parse_node::{MClassNode, NodeInfo, OpNode, ParseNode, ParseNodeType, SupSubNode},
    symbols::Atom,
    util, Options,
};

#[cfg(feature = "html")]
use crate::dom_tree::{CssStyle, HtmlNode};
#[cfg(feature = "mathml")]
use crate::mathml_tree::MathmlNode;

use super::{ord_argument, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let math = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::MClass, 1).with_primitive(true),
        handler: Box::new(|ctx, args, _opt_args| {
            let body = args[0].clone();
            ParseNode::MClass(MClassNode {
                m_class: format!("m{}", ctx.func_name),
                is_character_box: util::is_character_box(&body),
                body: ord_argument(body),
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });

    fns.insert_for_all_str(
        [
            "\\mathord",
            "\\mathbin",
            "\\mathrel",
            "\\mathopen",
            "\\mathclose",
            "\\mathpunct",
            "\\mathinner",
        ]
        .into_iter(),
        math,
    );

    let binrel = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::MClass, 2),
        handler: Box::new(|ctx, args, _opt_args| {
            ParseNode::MClass(MClassNode {
                m_class: bin_rel_class(&args[0]),
                body: ord_argument(args[1].clone()),
                is_character_box: util::is_character_box(&args[1]),
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });

    fns.insert(Cow::Borrowed("\\@binrel"), binrel);

    let stack = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::MClass, 2),
        handler: Box::new(|ctx, args, _opt_args| {
            let base_arg = &args[1];
            let shifted_arg = &args[0];

            let m_class = if ctx.func_name != "\\stackrel" {
                // LaTeX applies \binrel spacing to \overset and \underset
                bin_rel_class(base_arg)
            } else {
                "mrel".to_string()
            };

            let base_op = OpNode {
                limits: true,
                always_handle_sup_sub: Some(true),
                suppress_base_shift: Some(ctx.func_name != "\\stackrel"),
                parent_is_sup_sub: Some(false),
                symbol: false,
                name: None,
                body: Some(ord_argument(base_arg.clone())),
                info: NodeInfo::new_mode(base_arg.info().mode),
            };

            let sup_sub = SupSubNode {
                base: Some(Box::new(ParseNode::Op(base_op.clone()))),
                sup: if ctx.func_name == "\\underset" {
                    None
                } else {
                    Some(Box::new(shifted_arg.clone()))
                },
                sub: if ctx.func_name == "\\overset" {
                    Some(Box::new(shifted_arg.clone()))
                } else {
                    None
                },
                info: NodeInfo::new_mode(shifted_arg.info().mode),
            };
            let sup_sub = ParseNode::SupSub(sup_sub);

            ParseNode::MClass(MClassNode {
                m_class,
                is_character_box: util::is_character_box(&sup_sub),
                body: vec![sup_sub],
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });

    fns.insert_for_all_str(["\\stackrel", "\\overset", "\\underset"].into_iter(), stack);
}

fn bin_rel_class(arg: &ParseNode) -> String {
    // \binrel@ spacing varies with (bin|rel|ord) of the atom in the argument.
    // (by rendering separately and with {}s before and after, and measuring
    // the change in spacing).  We'll do roughly the same by detecting the
    // atom type directly.
    let atom = if let ParseNode::OrdGroup(ord) = arg {
        if !ord.body.is_empty() {
            &ord.body[0]
        } else {
            arg
        }
    } else {
        arg
    };
    match atom {
        ParseNode::Atom(atom) if atom.family == Atom::Bin || atom.family == Atom::Rel => {
            format!("m{}", atom.family.as_str())
        }
        _ => "mord".to_string(),
    }
}

#[cfg(feature = "html")]
fn html_builder(group: &ParseNode, options: &Options) -> HtmlNode {
    let ParseNode::MClass(group) = group else {
        panic!()
    };
    let elements =
        html::build_expression(&group.body, options, html::RealGroup::Root, (None, None));

    make_span(
        vec![group.m_class.to_string()],
        elements,
        Some(options),
        CssStyle::default(),
    )
    .into()
}

#[cfg(feature = "mathml")]
fn mathml_builder(group: &ParseNode, options: &Options) -> MathmlNode {
    use crate::{
        mathml,
        mathml_tree::{MathNode, MathNodeType},
        tree::ClassList,
    };

    let ParseNode::MClass(group) = group else {
        panic!()
    };

    let inner = mathml::build_expression(&group.body, options, None);

    match group.m_class.as_str() {
        "minner" => MathNode::new(MathNodeType::MPadded, inner, ClassList::new()),
        "mord" => {
            if group.is_character_box {
                let node = inner.into_iter().nth(0).unwrap();
                let MathmlNode::Math(mut node) = node else {
                    panic!()
                };

                node.typ = MathNodeType::Mi;

                node
            } else {
                MathNode::new(MathNodeType::Mi, inner, ClassList::new())
            }
        }
        _ => {
            let mut node = if group.is_character_box {
                let node = inner.into_iter().nth(0).unwrap();
                let MathmlNode::Math(mut node) = node else {
                    panic!()
                };

                node.typ = MathNodeType::Mo;

                node
            } else {
                MathNode::new(MathNodeType::Mo, inner, ClassList::new())
            };

            // Set spacing based on what is the most likely adjacent atom type.
            // See TeXbook p170.
            match group.m_class.as_str() {
                "mbin" => {
                    // medium space
                    node.set_attribute("lspace", "0.22em");
                    node.set_attribute("rspace", "0.22em");
                }
                "mpunct" => {
                    node.set_attribute("lspace", "0em");
                    // thin space
                    node.set_attribute("rspace", "0.16em");
                }
                "mopen" | "mclose" => {
                    node.set_attribute("lspace", "0em");
                    node.set_attribute("rspace", "0em");
                }
                "minner" => {
                    // 1mu is the most likely option
                    node.set_attribute("lspace", "0.0556em");
                    node.set_attribute("width", "+0.1111em");
                }
                _ => {}
            }

            // MathML <mo> default space is 5/18 em, so <mrel> needs no action.
            // Ref: https://developer.mozilla.org/en-US/docs/Web/MathML/Element/mo

            node
        }
    }
    .into()
}
