use std::borrow::Cow;
use std::sync::Arc;

use crate::parse_node::{MathChoiceNode, NodeInfo, ParseNode, ParseNodeType};
use crate::parser::ParseError;
use crate::style::{DISPLAY_STYLE, SCRIPT_SCRIPT_STYLE, SCRIPT_STYLE, TEXT_STYLE};

use super::{ord_argument, FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

#[cfg(feature = "html")]
use crate::{build_common, html};
#[cfg(feature = "mathml")]
use crate::mathml;

/// Choose the appropriate body based on the current math style
fn choose_math_style<'a>(group: &'a MathChoiceNode, options: &crate::Options) -> &'a [ParseNode] {
    match options.style.size() {
        s if s == DISPLAY_STYLE.size() => &group.display,
        s if s == TEXT_STYLE.size() => &group.text,
        s if s == SCRIPT_STYLE.size() => &group.script,
        s if s == SCRIPT_SCRIPT_STYLE.size() => &group.script_script,
        _ => &group.text,
    }
}

pub fn add_functions(fns: &mut Functions) {
    let mathchoice = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::MathChoice, 4)
            .with_primitive(true)
            .with_allowed_in_argument(true),
        handler: Box::new(mathchoice_handler),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            let ParseNode::MathChoice(mathchoice) = group else { panic!() };

            let body = choose_math_style(mathchoice, options);
            let elements = html::build_expression(body, options, html::RealGroup::False, (None, None));
            build_common::make_fragment(elements).into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            let ParseNode::MathChoice(mathchoice) = group else { panic!() };

            let body = choose_math_style(mathchoice, options);
            mathml::build_expression_row(body, options, None)
        })),
    });
    fns.insert(Cow::Borrowed("\\mathchoice"), mathchoice);
}

fn mathchoice_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::MathChoice(MathChoiceNode {
        display: ord_argument(args[0].clone()),
        text: ord_argument(args[1].clone()),
        script: ord_argument(args[2].clone()),
        script_script: ord_argument(args[3].clone()),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    }))
}
