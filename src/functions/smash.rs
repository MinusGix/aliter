use std::borrow::Cow;
use std::sync::Arc;

use crate::parse_node::{NodeInfo, ParseNode, ParseNodeType, SmashNode};
use crate::parser::ParseError;

use super::{FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

#[cfg(feature = "html")]
use crate::{build_common, dom_tree::{CssStyle, WithHtmlDomNode}, html};
#[cfg(feature = "mathml")]
use crate::{mathml, mathml_tree::{MathNode, MathNodeType}};
#[cfg(any(feature = "html", feature = "mathml"))]
use crate::tree::ClassList;

pub fn add_functions(fns: &mut Functions) {
    let smash = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_opt_args(ParseNodeType::Smash, 1, 1)
            .with_allowed_in_text(true),
        handler: Box::new(smash_handler),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            let ParseNode::Smash(smash) = group else { panic!() };

            let mut node = build_common::make_span(
                Vec::new(),
                vec![html::build_group(Some(&smash.body), options, None)],
                Some(options),
                CssStyle::default(),
            );

            if !smash.smash_height && !smash.smash_depth {
                return node.into();
            }

            if smash.smash_height {
                node.node.height = 0.0;
                // In order to influence makeVList, we have to reset the children.
                for child in &mut node.children {
                    child.node_mut().height = 0.0;
                }
            }

            if smash.smash_depth {
                node.node.depth = 0.0;
                for child in &mut node.children {
                    child.node_mut().depth = 0.0;
                }
            }

            // At this point, we've reset the TeX-like height and depth values.
            // But the span still has an HTML line height.
            // makeVList applies "display: table-cell", which prevents the browser
            // from acting on that line height. So we'll call makeVList now.
            let smashed_node = build_common::make_v_list(
                build_common::VListParam::FirstBaseLine {
                    children: vec![build_common::VListShiftChild::Elem(
                        build_common::VListElem::new(node),
                    )],
                },
                options,
            );

            // For spacing, TeX treats \smash as a math group (same spacing as ord).
            build_common::make_span::<crate::dom_tree::HtmlNode>(
                vec!["mord".to_string()],
                vec![smashed_node.into()],
                Some(options),
                CssStyle::default(),
            )
            .into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            let ParseNode::Smash(smash) = group else { panic!() };
            let inner = mathml::build_group(Some(&smash.body), options);
            let mut node = MathNode::<crate::mathml_tree::MathmlNode>::new(
                MathNodeType::MPadded,
                vec![inner],
                ClassList::new(),
            );

            if smash.smash_height {
                node.set_attribute("height", "0px");
            }

            if smash.smash_depth {
                node.set_attribute("depth", "0px");
            }

            node.into()
        })),
    });
    fns.insert(Cow::Borrowed("\\smash"), smash);
}

fn smash_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let mut smash_height = false;
    let mut smash_depth = false;

    if let Some(Some(tb_arg)) = opt_args.first() {
        // Optional [tb] argument is engaged.
        // ref: amsmath: \renewcommand{\smash}[1][tb]{%
        //               def\mb@t{\ht}\def\mb@b{\dp}\def\mb@tb{\ht\z@\z@\dp}%
        if let ParseNode::OrdGroup(group) = tb_arg {
            for node in &group.body {
                // Get the text of each node
                let letter = match node {
                    ParseNode::TextOrd(t) => Some(t.text.as_ref()),
                    ParseNode::MathOrd(m) => Some(m.text.as_ref()),
                    _ => None,
                };

                match letter {
                    Some("t") => smash_height = true,
                    Some("b") => smash_depth = true,
                    _ => {
                        // Invalid character, reset both
                        smash_height = false;
                        smash_depth = false;
                        break;
                    }
                }
            }
        }
    } else {
        // No optional argument means smash both height and depth
        smash_height = true;
        smash_depth = true;
    }

    Ok(ParseNode::Smash(SmashNode {
        body: Box::new(args[0].clone()),
        smash_height,
        smash_depth,
        info: NodeInfo::new_mode(ctx.parser.mode()),
    }))
}
