use std::{borrow::Cow, sync::Arc};

#[cfg(feature = "html")]
use crate::dom_tree::WithHtmlDomNode;
#[cfg(feature = "mathml")]
use crate::mathml_tree::WithMathDomNode;
use crate::{
    build_common::{make_line_span, make_span, make_v_list, VListElemShift, VListParam},
    delimiter,
    dom_tree::CssStyle,
    expander::Mode,
    html,
    lexer::Token,
    parse_node::{GenFracNode, InfixNode, NodeInfo, ParseNode, ParseNodeType},
    style::{StyleId, DISPLAY_STYLE, SCRIPT_SCRIPT_STYLE, SCRIPT_STYLE, TEXT_STYLE},
    symbols::Atom,
    unit::calculate_size,
    util::{ArgType, Style, StyleAuto},
    Options,
};

use super::{normalize_argument, FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let genfrac = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::GenFrac, 2)
            .with_allowed_in_argument(true),
        handler: Box::new(genfrac_handler),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });

    fns.insert_for_all_str(GENFRAC_NAMES.iter().copied(), genfrac);

    let gen_cfrac = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::GenFrac, 2),
        handler: Box::new(genfrac_cfrac_handler),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });

    fns.insert(Cow::Borrowed("\\cfrac"), gen_cfrac);

    let infix = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Infix, 0).with_infix(true),
        handler: Box::new(infix_handler),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
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
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });

    fns.insert(Cow::Borrowed("\\genfrac"), genfrac2);

    let infix_above = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Infix, 1)
            .with_arg_types(&[ArgType::Size] as &[_])
            .with_infix(true),
        handler: Box::new(infix_above_handler),
        // TODO:
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
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
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
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

fn adjust_style(size: &StyleAuto, original_style: StyleId) -> StyleId {
    // Figure out what style this fraction should be in based on the function used
    match size {
        StyleAuto::Style(style) => match style {
            Style::Text if original_style.size() == DISPLAY_STYLE.size() => {
                // We're in a \tfrac but incoming style is displaystyle
                TEXT_STYLE
            }
            Style::Display => {
                if original_style.as_id() >= SCRIPT_STYLE.as_id() {
                    original_style.text()
                } else {
                    DISPLAY_STYLE
                }
            }
            Style::Script => SCRIPT_STYLE,
            Style::ScriptScript => SCRIPT_SCRIPT_STYLE,
            _ => original_style,
        },
        StyleAuto::Auto => original_style,
    }
}

// TODO: should we just have this accept a `&GenFracNode`? Though we'd need a wrapper function to convert..
#[cfg(feature = "html")]
fn html_builder(node: &ParseNode, options: &Options) -> Box<dyn WithHtmlDomNode> {
    use crate::tree::ClassList;

    let ParseNode::GenFrac(group) = node else {
        // TODO: Don't panic
        panic!()
    };

    let style = adjust_style(&group.size, options.style);

    let nstyle = style.frac_num();
    let dstyle = style.frac_den();

    let new_options = options.having_style(nstyle);
    let new_options = new_options.as_ref().unwrap_or(options);

    let mut numerm = html::build_group(Some(&group.numer), new_options, Some(options));

    if group.continued {
        // \cfrac inserts a \strut into the numerator
        // Get \strut dimensions from TeXbook page 353
        let h_strut = 8.5 / options.font_metrics().pt_per_em;
        let d_strut = 3.5 / options.font_metrics().pt_per_em;

        let height = numerm.node().height;
        let depth = numerm.node().depth;
        numerm.node_mut().height = height.max(h_strut);
        numerm.node_mut().depth = depth.max(d_strut);
    }

    let new_options = options.having_style(dstyle);
    let new_options = new_options.as_ref().unwrap_or(options);

    let denomm = html::build_group(Some(&group.denom), new_options, Some(options));

    let (rule, rule_width, rule_spacing) = if group.has_bar_line {
        let rule = if let Some(bar_size) = &group.bar_size {
            let rule_width = calculate_size(bar_size, options);
            make_line_span("frac-line", options, Some(rule_width))
        } else {
            make_line_span("frac-line", options, None)
        };
        let height = rule.node.height;

        (Some(rule), height, height)
    } else {
        (None, 0.0, options.font_metrics().default_rule_thickness)
    };

    // Rule 15b
    let mut num_shift;
    let clearance;
    let mut denom_shift;
    if style.size() == DISPLAY_STYLE.size() || group.size == StyleAuto::Style(Style::Display) {
        num_shift = options.font_metrics().num1;
        clearance = if rule_width > 0.0 {
            3.0 * rule_spacing
        } else {
            7.0 * rule_spacing
        };
        denom_shift = options.font_metrics().denom1;
    } else {
        if rule_width > 0.0 {
            num_shift = options.font_metrics().num2;
            clearance = rule_spacing;
        } else {
            num_shift = options.font_metrics().num3;
            clearance = 3.0 * rule_spacing;
        }
        denom_shift = options.font_metrics().denom2;
    }

    // Note that these sections are swapped in the order from KaTeX, since this is more natural in rust
    let mut frac = if let Some(rule) = rule {
        // Rule 15d
        let axis_height = options.font_metrics().axis_height;

        let num_clearance_shift =
            (num_shift - numerm.node().depth) - (axis_height + 0.5 * rule_width);
        if num_clearance_shift < clearance {
            num_shift += clearance - num_clearance_shift;
        }

        let denom_clearance_shift =
            (axis_height - 0.5 * rule_width) - (denomm.node().height - denom_shift);
        if denom_clearance_shift < clearance {
            denom_shift += clearance - denom_clearance_shift;
        }

        let mid_shift = -(axis_height - 0.5 * rule_width);

        let rule = Box::new(rule);
        make_v_list(
            VListParam::IndividualShift {
                children: vec![
                    VListElemShift::new(denomm, denom_shift),
                    VListElemShift::new(rule, mid_shift),
                    VListElemShift::new(numerm, -num_shift),
                ],
            },
            options,
        )
    } else {
        // Rule 15c
        let candidate_clearance =
            (num_shift - numerm.node().depth) - (denomm.node().height - denom_shift);
        if candidate_clearance < clearance {
            num_shift += 0.5 * (clearance - candidate_clearance);
            denom_shift += 0.5 * (clearance - candidate_clearance);
        }

        make_v_list(
            VListParam::IndividualShift {
                children: vec![
                    VListElemShift::new(denomm, denom_shift),
                    VListElemShift::new(numerm, -num_shift),
                ],
            },
            options,
        )
    };

    // Since we manually change the style sometimes (with \dfrac or \tfrac),
    // account for the possible size change here.
    let new_options = options.having_style(style);
    let new_options = new_options.as_ref().unwrap_or(options);

    frac.node.height *= new_options.size_multiplier() / options.size_multiplier();
    frac.node.depth *= new_options.size_multiplier() / options.size_multiplier();

    // Rule 15e
    let delim_size = if style.size() == DISPLAY_STYLE.size() {
        options.font_metrics().delim1
    } else if style.size() == SCRIPT_SCRIPT_STYLE.size() {
        options
            .having_style(SCRIPT_STYLE)
            .map(|x| x.font_metrics().delim2)
            .unwrap_or(options.font_metrics().delim2)
    } else {
        options.font_metrics().delim2
    };

    let left_delim = if let Some(left_delim) = &group.left_delim {
        let opts = options.having_style(style);
        let opts = opts.as_ref().unwrap_or(options);
        delimiter::custom_sized_delim(
            left_delim,
            delim_size,
            true,
            opts,
            group.info.mode,
            vec!["mopen".to_string()],
        )
    } else {
        html::make_null_delimiter(options, vec!["mopen".to_string()])
    };

    let right_delim = if group.continued {
        make_span(ClassList::new(), Vec::new(), None, CssStyle::default())
    } else if let Some(right_delim) = &group.right_delim {
        let opts = options.having_style(style);
        let opts = opts.as_ref().unwrap_or(options);
        delimiter::custom_sized_delim(
            &right_delim,
            delim_size,
            true,
            opts,
            group.info.mode,
            vec!["mclose".to_string()],
        )
    } else {
        html::make_null_delimiter(options, vec!["mclose".to_string()])
    };

    let classes: ClassList = ["mord".to_string()]
        .into_iter()
        .chain(new_options.sizing_classes(options))
        .collect();
    let children = vec![
        left_delim,
        make_span(
            vec!["mfrac".to_string()],
            vec![Box::new(frac)],
            None,
            CssStyle::default(),
        ),
        right_delim,
    ];

    Box::new(make_span(
        classes,
        children,
        Some(options),
        CssStyle::default(),
    ))
}

#[cfg(feature = "mathml")]
fn mathml_builder(node: &ParseNode, options: &Options) -> Box<dyn WithMathDomNode> {
    use crate::{
        mathml,
        mathml_tree::{self, MathNode, MathNodeType},
        tree::ClassList,
        unit::make_em,
    };

    let ParseNode::GenFrac(group) = node else {
        // TODO: Don't panic
        panic!()
    };

    let numer = mathml::build_group(Some(&group.numer), options);
    let denom = mathml::build_group(Some(&group.denom), options);
    let mut node = MathNode::new(MathNodeType::MFrac, vec![numer, denom], ClassList::new());

    if !group.has_bar_line {
        node.set_attribute("linethickness", "0px");
    } else if let Some(bar_size) = &group.bar_size {
        let rule_width = calculate_size(bar_size, options);
        node.set_attribute("linethickness", make_em(rule_width).as_str());
    }

    let style = adjust_style(&group.size, options.style);
    let node = if style.size() != options.style.size() {
        let is_display = style.size() == DISPLAY_STYLE.size();

        let node: Box<dyn WithMathDomNode> = Box::new(node);
        MathNode::new(MathNodeType::MStyle, vec![node], ClassList::new())
            .with_attribute("displaystyle", is_display.to_string())
            .with_attribute("scriptlevel", "0")
    } else {
        node
    };

    if group.left_delim.is_some() || group.right_delim.is_some() {
        let mut with_delims = Vec::new();

        if let Some(left_delim) = &group.left_delim {
            let left_text = left_delim.replace("\\", "");
            let left_text = mathml_tree::TextNode::new(left_text);
            let left_text: Box<dyn WithMathDomNode> = Box::new(left_text);

            let left_op = MathNode::new(MathNodeType::Mo, vec![left_text], ClassList::new())
                .with_attribute("fence", "true");

            with_delims.push(left_op);
        }

        with_delims.push(node);

        if let Some(right_delim) = &group.right_delim {
            let right_text = right_delim.replace("\\", "");
            let right_text = mathml_tree::TextNode::new(right_text);
            let right_text: Box<dyn WithMathDomNode> = Box::new(right_text);

            let right_op = MathNode::new(MathNodeType::Mo, vec![right_text], ClassList::new())
                .with_attribute("fence", "true");

            with_delims.push(right_op);
        }

        mathml::make_row(with_delims)
    } else {
        Box::new(node)
    }
}
