use std::sync::Arc;

use crate::{
    macr::MacroReplace,
    parse_node::{ColorNode, NodeInfo, ParseNode, ParseNodeType},
    util::ArgType,
};

use super::{ord_argument, FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let text_color = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Color, 2)
            .with_allowed_in_text(true)
            .with_arg_types(&[ArgType::Color, ArgType::Original] as &[ArgType]),
        handler: Box::new(text_color_handler),
    });

    fns.insert("\\textcolor".into(), text_color);

    let color = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Color, 1)
            .with_allowed_in_text(true)
            .with_arg_types(&[ArgType::Color] as &[ArgType]),
        handler: Box::new(color_handler),
    });

    fns.insert("\\color".into(), color);
}

fn text_color_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    let color = if let ParseNode::ColorToken(color) = &args[0] {
        color.color.clone()
    } else {
        // TODO: just return an error
        panic!();
    };

    let body = args[1].clone();

    ParseNode::Color(ColorNode {
        color,
        body: ord_argument(body),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}

fn color_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    let color = if let ParseNode::ColorToken(color) = &args[0] {
        color.color.clone()
    } else {
        // TODO: just return an error
        panic!();
    };

    // Set the macro \current@color in the current namespace to store the current color
    // mimicking the behavior of color.sty
    // This is currently used just to correctly color a \right that follows a \color command
    ctx.parser.gullet.macros.set_back_macro(
        "\\current@color".to_string(),
        Some(Arc::new(MacroReplace::Text(color.to_string()))),
    );

    // Parse out the implicit body that should be colored
    let body = ctx
        .parser
        .dispatch_parse_expression(true, ctx.break_on_token_text)
        .unwrap();

    ParseNode::Color(ColorNode {
        color,
        body,
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}
