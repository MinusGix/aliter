use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::parse_node::{
    DelimSize, DelimSizingNode, LeftRightNode, LeftRightRightNode, MClass, MiddleNode, NodeInfo,
    ParseNode, ParseNodeType,
};
use crate::parser::ParseError;

use super::{FunctionPropSpec, FunctionSpec, Functions};

struct DelimInfo {
    m_class: MClass,
    size: DelimSize,
}

static DELIM_SIZE_MAP: Lazy<HashMap<&'static str, DelimInfo>> = Lazy::new(|| {
    use DelimSize::*;
    use MClass::*;
    HashMap::from([
        ("\\bigl", DelimInfo { m_class: Open, size: One }),
        ("\\Bigl", DelimInfo { m_class: Open, size: Two }),
        ("\\biggl", DelimInfo { m_class: Open, size: Three }),
        ("\\Biggl", DelimInfo { m_class: Open, size: Four }),
        ("\\bigr", DelimInfo { m_class: Close, size: One }),
        ("\\Bigr", DelimInfo { m_class: Close, size: Two }),
        ("\\biggr", DelimInfo { m_class: Close, size: Three }),
        ("\\Biggr", DelimInfo { m_class: Close, size: Four }),
        ("\\bigm", DelimInfo { m_class: MClass::Rel, size: One }),
        ("\\Bigm", DelimInfo { m_class: MClass::Rel, size: Two }),
        ("\\biggm", DelimInfo { m_class: MClass::Rel, size: Three }),
        ("\\Biggm", DelimInfo { m_class: MClass::Rel, size: Four }),
        ("\\big", DelimInfo { m_class: Ord, size: One }),
        ("\\Big", DelimInfo { m_class: Ord, size: Two }),
        ("\\bigg", DelimInfo { m_class: Ord, size: Three }),
        ("\\Bigg", DelimInfo { m_class: Ord, size: Four }),
    ])
});

const DELIMITERS: &'static [&'static str] = &[
    "(", "\\lparen", ")", "\\rparen", "[", "\\lbrack", "]", "\\rbrack", "\\{", "\\lbrace",
    "\\}", "\\rbrace", "\\lfloor", "\\rfloor", "\u{230a}", "\u{230b}", "\\lceil", "\\rceil",
    "\u{2308}", "\u{2309}", "<", ">", "\\langle", "\u{27e8}", "\\rangle", "\u{27e9}", "\\lt",
    "\\gt", "\\lvert", "\\rvert", "\\lVert", "\\rVert", "\\lgroup", "\\rgroup", "\u{27ee}",
    "\u{27ef}", "\\lmoustache", "\\rmoustache", "\u{23b0}", "\u{23b1}", "/", "\\backslash", "|",
    "\\vert", "\\|", "\\Vert", "\\uparrow", "\\Uparrow", "\\downarrow", "\\Downarrow",
    "\\updownarrow", "\\Updownarrow", ".",
];

fn canonical_delim(delim: &str) -> String {
    match delim {
        "\\langle" | "\\lt" | "<" => "⟨".to_string(),
        "\\rangle" | "\\gt" | ">" => "⟩".to_string(),
        "\\lbrace" | "\\{" => "{".to_string(),
        "\\rbrace" | "\\}" => "}".to_string(),
        "\\lparen" => "(".to_string(),
        "\\rparen" => ")".to_string(),
        "\\lbrack" => "[".to_string(),
        "\\rbrack" => "]".to_string(),
        "\\vert" | "\\lvert" | "\\rvert" | "|" => "|".to_string(),
        "\\|" | "\\lVert" | "\\rVert" => "||".to_string(),
        "\\backslash" => "\\".to_string(),
        other => other.to_string(),
    }
}

/// Check if the given node is a valid delimiter.
/// In KaTeX, this is done by checkSymbolNodeType which accepts both
/// atom nodes and non-atom symbol nodes (textord, mathord, etc.)
fn check_delimiter(arg: &ParseNode) -> Result<String, ParseError> {
    // Get the text from the node (supports atom, textord, mathord, etc.)
    let delim = match arg {
        ParseNode::Atom(atom) => atom.text.as_ref(),
        ParseNode::TextOrd(ord) => ord.text.as_ref(),
        ParseNode::MathOrd(ord) => ord.text.as_ref(),
        _ => return Err(ParseError::Expected),
    };

    if DELIMITERS.contains(&delim) {
        Ok(canonical_delim(delim))
    } else {
        Err(ParseError::Expected)
    }
}

pub fn add_functions(fns: &mut Functions) {
    // Delimiter sizing commands
    let delim_sizing = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::DelimSizing, 1)
            .with_arg_types(&[crate::util::ArgType::Primitive] as &[crate::util::ArgType]),
        handler: Box::new(move |ctx, args, _| {
            let delim = check_delimiter(&args[0]).unwrap();
            let info = DELIM_SIZE_MAP.get(ctx.func_name.as_ref()).unwrap();

            ParseNode::DelimSizing(DelimSizingNode {
                size: info.size,
                m_class: info.m_class,
                delim: Cow::Owned(delim),
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });
    for name in DELIM_SIZE_MAP.keys() {
        fns.insert(Cow::Borrowed(*name), delim_sizing.clone());
    }

    // \right
    let right = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::LeftRightRight, 1)
            .with_primitive(true)
            .with_allowed_in_argument(true),
        handler: Box::new(|ctx, args, _| {
            let delim = check_delimiter(&args[0]).unwrap();
            ParseNode::LeftRightRight(LeftRightRightNode {
                delim,
                color: None,
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });
    fns.insert(Cow::Borrowed("\\right"), right);

    // \left
    let left = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::LeftRight, 1)
            .with_primitive(true)
            .with_allowed_in_argument(true),
        handler: Box::new(|ctx, args, _| {
            let left_delim = check_delimiter(&args[0]).unwrap();

            // Track nesting depth for \middle validation
            ctx.parser.leftright_depth += 1;

            let body =
                ctx.parser
                    .dispatch_parse_expression(false, Some(crate::expander::BreakToken::Right))
                    .unwrap();

            ctx.parser.leftright_depth -= 1;

            // Parse the following \right
            let right = ctx
                .parser
                .parse_function(Some(crate::expander::BreakToken::Right), None)
                .unwrap()
                .expect("Expected \\right");
            let ParseNode::LeftRightRight(right) = right else {
                panic!("Expected LeftRightRight node");
            };

            ParseNode::LeftRight(LeftRightNode {
                left: left_delim,
                right: right.delim,
                right_color: right.color,
                body,
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });
    fns.insert(Cow::Borrowed("\\left"), left);

    // \middle
    let middle = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Middle, 1)
            .with_primitive(true),
        handler: Box::new(|ctx, args, _| {
            let delim = check_delimiter(&args[0]).unwrap();

            if ctx.parser.leftright_depth == 0 {
                panic!("\\middle without preceding \\left");
            }

            ParseNode::Middle(MiddleNode {
                delim,
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });
    fns.insert(Cow::Borrowed("\\middle"), middle);
}
