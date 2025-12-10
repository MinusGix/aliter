use std::borrow::Cow;
use std::sync::Arc;

use crate::parse_node::{HorizBraceNode, NodeInfo, ParseNode, ParseNodeType};
use crate::parser::ParseError;

use super::{FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let horiz_brace = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::HorizBrace, 1),
        handler: Box::new(horiz_brace_handler),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            use crate::{
                build_common::{make_span, make_v_list, VListElem, VListKern, VListParam, VListShiftChild},
                dom_tree::{CssStyle, HtmlNode, WithHtmlDomNode},
                html,
                stretchy,
                style::DISPLAY_STYLE,
            };

            let ParseNode::HorizBrace(group) = group else {
                panic!("Expected HorizBrace node");
            };

            // Build the base group
            let base_options = options.having_base_style(Some(DISPLAY_STYLE)).unwrap_or_else(|| options.clone());
            let body = html::build_group(Some(&group.base), &base_options, None);

            // Create the stretchy brace SVG
            let brace_body = stretchy::svg_span(&group.label, options);

            // Generate the vlist with content and brace
            let vlist = if group.is_over {
                make_v_list(
                    VListParam::FirstBaseLine {
                        children: vec![
                            VListShiftChild::Elem(VListElem::new(body)),
                            VListShiftChild::Kern(VListKern(0.1)),
                            VListShiftChild::Elem(VListElem::new_with_wrapper_classes(brace_body, vec!["svg-align".to_string()])),
                        ],
                    },
                    options,
                )
            } else {
                let body_depth = body.node().depth;
                let brace_height = brace_body.node().height;
                make_v_list(
                    VListParam::Bottom {
                        amount: body_depth + 0.1 + brace_height,
                        children: vec![
                            VListShiftChild::Elem(VListElem::new_with_wrapper_classes(brace_body, vec!["svg-align".to_string()])),
                            VListShiftChild::Kern(VListKern(0.1)),
                            VListShiftChild::Elem(VListElem::new(body)),
                        ],
                    },
                    options,
                )
            };

            let mord_class = if group.is_over { "mover" } else { "munder" };
            make_span::<HtmlNode>(
                vec!["mord".to_string(), mord_class.to_string()],
                vec![vlist.into()],
                Some(options),
                CssStyle::default(),
            ).into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            use crate::{mathml, mathml_tree::{MathNode, MathNodeType}, stretchy, tree::ClassList};

            let ParseNode::HorizBrace(group) = group else {
                panic!("Expected HorizBrace node");
            };

            let accent_node = stretchy::mathml_node(&group.label);
            let base_node = mathml::build_group(Some(&group.base), options);

            let node_type = if group.is_over {
                MathNodeType::MOver
            } else {
                MathNodeType::MUnder
            };

            MathNode::new(node_type, vec![base_node, accent_node.into()], ClassList::new()).into()
        })),
    });

    fns.insert(Cow::Borrowed("\\overbrace"), horiz_brace.clone());
    fns.insert(Cow::Borrowed("\\underbrace"), horiz_brace);
}

fn horiz_brace_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let func_name = ctx.func_name.into_owned();
    Ok(ParseNode::HorizBrace(HorizBraceNode {
        label: func_name.clone(),
        is_over: func_name.starts_with("\\over"),
        base: Box::new(args[0].clone()),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    }))
}
