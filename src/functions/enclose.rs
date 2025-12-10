use std::borrow::Cow;
use std::sync::Arc;

use crate::parse_node::{EncloseNode, NodeInfo, ParseNode, ParseNodeType};
use crate::parser::ParseError;
use crate::util::ArgType;

use super::{FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    // \colorbox
    let colorbox = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Enclose, 2)
            .with_allowed_in_text(true)
            .with_arg_types(&[ArgType::Color, ArgType::Mode(crate::expander::Mode::Text)] as &[ArgType]),
        handler: Box::new(colorbox_handler),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });
    fns.insert(Cow::Borrowed("\\colorbox"), colorbox);

    // \fcolorbox
    let fcolorbox = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Enclose, 3)
            .with_allowed_in_text(true)
            .with_arg_types(
                &[ArgType::Color, ArgType::Color, ArgType::Mode(crate::expander::Mode::Text)]
                    as &[ArgType],
            ),
        handler: Box::new(fcolorbox_handler),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });
    fns.insert(Cow::Borrowed("\\fcolorbox"), fcolorbox);

    // \fbox
    let fbox = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Enclose, 1)
            .with_allowed_in_text(true)
            .with_arg_types(&[ArgType::HBox] as &[ArgType]),
        handler: Box::new(fbox_handler),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });
    fns.insert(Cow::Borrowed("\\fbox"), fbox);

    // \boxed (math)
    let boxed = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Enclose, 1)
            .with_allowed_in_text(true),
        handler: Box::new(boxed_handler),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });
    fns.insert(Cow::Borrowed("\\boxed"), boxed);

    // Cancel/strike/phase
    let cancel = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Enclose, 1),
        handler: Box::new(cancel_handler),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });
    fns.insert(Cow::Borrowed("\\cancel"), cancel.clone());
    fns.insert(Cow::Borrowed("\\bcancel"), cancel.clone());
    fns.insert(Cow::Borrowed("\\xcancel"), cancel.clone());
    fns.insert(Cow::Borrowed("\\sout"), cancel.clone());
    fns.insert(Cow::Borrowed("\\phase"), cancel);

    // \angl
    let angl = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Enclose, 1)
            .with_arg_types(&[ArgType::HBox] as &[ArgType]),
        handler: Box::new(|ctx, args, _| {
            Ok(ParseNode::Enclose(EncloseNode {
                label: "\\angl".to_string(),
                background_color: None,
                border_color: None,
                body: Box::new(args[0].clone()),
                info: NodeInfo::new_mode(ctx.parser.mode()),
            }))
        }),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });
    fns.insert(Cow::Borrowed("\\angl"), angl);
}

fn colorbox_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let color = match &args[0] {
        ParseNode::ColorToken(tok) => tok.color.clone(),
        _ => panic!("Expected ColorToken"),
    };

    Ok(ParseNode::Enclose(EncloseNode {
        label: ctx.func_name.into_owned(),
        background_color: Some(color),
        border_color: None,
        body: Box::new(args[1].clone()),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    }))
}

fn fcolorbox_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let border_color = match &args[0] {
        ParseNode::ColorToken(tok) => tok.color.clone(),
        _ => panic!("Expected ColorToken"),
    };
    let background_color = match &args[1] {
        ParseNode::ColorToken(tok) => tok.color.clone(),
        _ => panic!("Expected ColorToken"),
    };

    Ok(ParseNode::Enclose(EncloseNode {
        label: ctx.func_name.into_owned(),
        background_color: Some(background_color),
        border_color: Some(border_color),
        body: Box::new(args[2].clone()),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    }))
}

fn fbox_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::Enclose(EncloseNode {
        label: "\\fbox".to_string(),
        background_color: None,
        border_color: None,
        body: Box::new(args[0].clone()),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    }))
}

fn boxed_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::Enclose(EncloseNode {
        label: "\\boxed".to_string(),
        background_color: None,
        border_color: None,
        body: Box::new(args[0].clone()),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    }))
}

fn cancel_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::Enclose(EncloseNode {
        label: ctx.func_name.into_owned(),
        background_color: None,
        border_color: None,
        body: Box::new(args[0].clone()),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    }))
}
