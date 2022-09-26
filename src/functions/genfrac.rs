use std::{borrow::Cow, sync::Arc};

use crate::{
    expander::Mode,
    lexer::Token,
    parse_node::{GenFracNode, InfixNode, NodeInfo, ParseNode, ParseNodeType},
    symbols::Atom,
    util::{ArgType, Style, StyleAuto},
};

use super::{normalize_argument, FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let genfrac = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::GenFrac, 2)
            .with_allowed_in_argument(true),
        handler: Box::new(genfrac_handler),
    });

    fns.insert_for_all_str(GENFRAC_NAMES.iter().copied(), genfrac);

    let gen_cfrac = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::GenFrac, 2),
        handler: Box::new(genfrac_cfrac_handler),
    });

    fns.insert(Cow::Borrowed("\\cfrac"), gen_cfrac);

    let infix = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Infix, 0).with_infix(true),
        handler: Box::new(infix_handler),
    });

    fns.insert_for_all_str(INFIX_NAMES.iter().copied(), infix);

    let genfrac2 = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::GenFrac, 6)
            .with_allowed_in_argument(true)
            .with_arg_types(&[
                ArgType::Mode(Mode::Math),
                ArgType::Mode(Mode::Math),
                ArgType::Size,
                ArgType::Mode(Mode::Text),
                ArgType::Mode(Mode::Math),
                ArgType::Mode(Mode::Math),
            ] as &[_]),
        handler: Box::new(genfrac2_handler),
    });

    fns.insert(Cow::Borrowed("\\genfrac"), genfrac2);

    let infix_above = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Infix, 1)
            .with_arg_types(&[ArgType::Size] as &[_])
            .with_infix(true),
        handler: Box::new(infix_above_handler),
    });

    fns.insert(Cow::Borrowed("\\above"), infix_above);

    let genfrac_abovefrac = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::GenFrac, 3).with_arg_types(&[
            ArgType::Mode(Mode::Math),
            ArgType::Size,
            ArgType::Mode(Mode::Math),
        ]
            as &[_]),
        handler: Box::new(genfrac_abovefrac_handler),
    });

    fns.insert(Cow::Borrowed("\\\\abovefrac"), genfrac_abovefrac);
}

const GENFRAC_NAMES: &'static [&'static str] = &[
    "\\dfrac",
    "\\frac",
    "\\tfrac",
    "\\dbinom",
    "\\binom",
    "\\tbinom",
    // can’t be entered directly
    "\\\\atopfrac",
    "\\\\bracefrac",
    "\\\\brackfrac",
];

fn genfrac_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    let numer = Box::new(args[0].clone());
    let denom = Box::new(args[1].clone());

    let has_bar_line;
    let mut left_delim = None;
    let mut right_delim = None;

    match ctx.func_name.as_ref() {
        "\\dfrac" | "\\frac" | "\\tfrac" => has_bar_line = true,
        "\\\\atopfrac" => has_bar_line = false,
        "\\dbinom" | "\\binom" | "\\tbinom" => {
            has_bar_line = false;
            left_delim = Some("(");
            right_delim = Some(")");
        }
        "\\\\bracefrac" => {
            has_bar_line = false;
            left_delim = Some("\\{");
            right_delim = Some("\\}");
        }
        "\\\\brackfrac" => {
            has_bar_line = false;
            left_delim = Some("\\[");
            right_delim = Some("\\]");
        }
        // TODO: Don't panic
        _ => panic!("Unrecognized genfrac command"),
    }

    let size = match ctx.func_name.as_ref() {
        "\\dfrac" | "\\dbinom" => StyleAuto::Style(Style::Display),
        "\\tfrac" | "\\tbinom" => StyleAuto::Style(Style::Text),
        _ => StyleAuto::Auto,
    };

    ParseNode::GenFrac(GenFracNode {
        continued: false,
        numer,
        denom,
        has_bar_line,
        left_delim: left_delim.map(Cow::Borrowed),
        right_delim: right_delim.map(Cow::Borrowed),
        size,
        bar_size: None,
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}

fn genfrac_cfrac_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    let numer = Box::new(args[0].clone());
    let denom = Box::new(args[1].clone());

    ParseNode::GenFrac(GenFracNode {
        continued: true,
        numer,
        denom,
        has_bar_line: true,
        left_delim: None,
        right_delim: None,
        size: StyleAuto::Style(Style::Display),
        bar_size: None,
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}

const INFIX_NAMES: &'static [&'static str] =
    &["\\over", "\\choose", "\\atop", "\\brace", "\\brack"];

fn infix_handler(
    ctx: FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    let replace_with = match ctx.func_name.as_ref() {
        "\\over" => "\\frac",
        "\\choose" => "\\binom",
        "\\atop" => "\\\\atopfrac",
        "\\brace" => "\\\\bracefrac",
        "\\brack" => "\\\\brackfrac",
        _ => unreachable!("Unrecognized infix genfrac command"),
    };

    ParseNode::Infix(InfixNode {
        replace_with: Cow::Borrowed(replace_with),
        size: None,
        token: ctx.token.map(Token::into_owned),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}

fn delim_from_value(delim: &str) -> Option<&str> {
    if delim.is_empty() || delim == "." {
        None
    } else {
        Some(delim)
    }
}

// TODO: Don't panic
fn style_from_num(text: &str) -> Style {
    let v = u64::from_str_radix(text, 10).unwrap();
    match v {
        0 => Style::Display,
        1 => Style::Text,
        2 => Style::Script,
        3 => Style::ScriptScript,
        _ => panic!(),
    }
}

fn genfrac2_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    let numer = Box::new(args[4].clone());
    let denom = Box::new(args[5].clone());

    let left_node = normalize_argument(args[0].clone());
    let left_delim = if let ParseNode::Atom(atom) = &left_node {
        if atom.family == Atom::Open {
            delim_from_value(&atom.text)
        } else {
            None
        }
    } else {
        None
    };

    let right_node = normalize_argument(args[1].clone());
    let right_delim = if let ParseNode::Atom(atom) = &right_node {
        if atom.family == Atom::Close {
            delim_from_value(&atom.text)
        } else {
            None
        }
    } else {
        None
    };

    let bar_node = if let ParseNode::Size(size) = &args[2] {
        size.clone()
    } else {
        // TODO: Don't panic
        panic!();
    };

    let has_bar_line;
    let mut bar_size = None;

    if bar_node.is_blank {
        // \genfrac acts differently than \above
        // \genfrac treats an empty size group as a signal to use a
        // standard bar size. \above would see size = 0 and omit the bar.
        has_bar_line = true;
    } else {
        has_bar_line = bar_node.value.num() > 0.0;
        bar_size = Some(bar_node.value);
    }

    let size = match &args[3] {
        ParseNode::OrdGroup(ord) => {
            if ord.body.is_empty() {
                StyleAuto::Auto
            } else if let ParseNode::TextOrd(text_ord) = &ord.body[0] {
                StyleAuto::Style(style_from_num(&text_ord.text))
            } else {
                // TODO: Don't panic
                panic!();
            }
        }
        ParseNode::TextOrd(text_ord) => StyleAuto::Style(style_from_num(&text_ord.text)),
        // TODO: Don't panic:
        _ => panic!(),
    };

    ParseNode::GenFrac(GenFracNode {
        continued: false,
        numer,
        denom,
        has_bar_line,
        left_delim: left_delim.map(ToString::to_string).map(Cow::Owned),
        right_delim: right_delim.map(ToString::to_string).map(Cow::Owned),
        size,
        bar_size,
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}

fn infix_above_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    let size = if let ParseNode::Size(size) = &args[0] {
        size.value.clone()
    } else {
        panic!()
    };

    ParseNode::Infix(InfixNode {
        replace_with: Cow::Borrowed("\\\\abovefrac"),
        size: Some(size),
        token: ctx.token.map(Token::into_owned),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}

fn genfrac_abovefrac_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    let numer = Box::new(args[0].clone());
    let bar_size = if let ParseNode::Infix(infix) = &args[1] {
        infix.size.clone().unwrap()
    } else {
        panic!()
    };
    let denom = Box::new(args[2].clone());

    let has_bar_line = bar_size.num() > 0.0;

    ParseNode::GenFrac(GenFracNode {
        continued: false,
        numer,
        denom,
        has_bar_line,
        left_delim: None,
        right_delim: None,
        size: StyleAuto::Auto,
        bar_size: Some(bar_size),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}
