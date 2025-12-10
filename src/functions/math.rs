use std::borrow::Cow;
use std::sync::Arc;

use crate::expander::{BreakToken, Mode};
use crate::parse_node::{NodeInfo, ParseNode, ParseNodeType, StylingNode};
use crate::parser::ParseError;
use crate::util::Style;

use super::{FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    // Switching from text mode back to math mode: \( and $
    let math_delim = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Styling, 0)
            .with_allowed_in_text(true)
            .with_allowed_in_math(false),
        handler: Box::new(|ctx, _, _| {
            let outer_mode = ctx.parser.mode();
            ctx.parser.switch_mode(Mode::Math);

            let close = if ctx.func_name == "\\(" {
                BreakToken::BackslashRightParen
            } else {
                BreakToken::Dollar
            };

            let body = ctx
                .parser
                .dispatch_parse_expression(false, Some(close))
                .unwrap();

            // Consume the closing delimiter
            ctx.parser.expect(close.as_str(), true).unwrap();

            ctx.parser.switch_mode(outer_mode);

            Ok(ParseNode::Styling(StylingNode {
                style: Style::Text,
                body,
                info: NodeInfo::new_mode(ctx.parser.mode()),
            }))
        }),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });
    fns.insert(Cow::Borrowed("\\("), math_delim.clone());
    fns.insert(Cow::Borrowed("$"), math_delim);

    // Check for extra closing math delimiters: \) and \]
    let close_delim = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Styling, 0)
            .with_allowed_in_text(true)
            .with_allowed_in_math(false),
        handler: Box::new(|ctx, _, _| {
            panic!("Mismatched {}", ctx.func_name);
        }),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });
    fns.insert(Cow::Borrowed("\\)"), close_delim.clone());
    fns.insert(Cow::Borrowed("\\]"), close_delim);
}
