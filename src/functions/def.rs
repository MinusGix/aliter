use std::{borrow::Cow, sync::Arc};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    lexer::Token,
    macr::{MacroExpansion, MacroReplace},
    parse_node::{InternalNode, NodeInfo, ParseNode, ParseNodeType},
    parser::{ParseError, Parser},
};

use super::{FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let global_long = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Internal, 0).with_allowed_in_text(true),
        handler: Box::new(
            |ctx: FunctionContext, _args: &[ParseNode], _opt_args: &[Option<ParseNode>]| {
                ctx.parser.consume_spaces().unwrap();
                let mut token = ctx.parser.fetch_mut().unwrap();

                if let Some(map) = get_global_map(&token.content) {
                    if ctx.func_name == "\\global" || ctx.func_name == "\\\\globallong" {
                        token.content = Cow::Borrowed(map);
                    }

                    let func = ctx.parser.parse_function(None, None).unwrap();
                    if let Some(ParseNode::Internal(internal)) = func {
                        return ParseNode::Internal(internal);
                    } else {
                        panic!()
                    }
                }

                // TODO: don't panic
                panic!()
            },
        ),
        // TODO:
        #[cfg(feature = "html")]
        html_builder: None,
    });

    fns.insert_for_all_str(
        [
            "\\global",
            "\\long",
            "\\\\globallong", // can’t be entered directly
        ]
        .into_iter(),
        global_long,
    );

    let def = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Internal, 0)
            .with_allowed_in_text(true)
            .with_primitive(true),
        handler: Box::new(|ctx, _, _| {
            let token = ctx.parser.gullet.pop_token().unwrap();

            if CONTROL_SEQUENCE_REGEX.is_match(&token.content) {
                panic!();
            }

            let mut num_args = 0;
            let mut insert = None;
            let mut delimiters = vec![vec![]];

            while ctx.parser.gullet.future().unwrap().content != "{" {
                let tok = ctx.parser.gullet.pop_token().unwrap();
                if tok.content == "#" {
                    // If the very last character of the <parameter text> is #, so that
                    // this # is immediately followed by {, TeX will behave as if the {
                    // had been inserted at the right end of both the parameter text and the
                    // replacement text
                    if ctx.parser.gullet.future().unwrap().content == "{" {
                        insert = Some(ctx.parser.gullet.future().unwrap().clone().into_owned());
                        delimiters[num_args].push(Cow::Borrowed("{"));
                        break;
                    }

                    // A parameter ,the first appearance of # must be followed by 1
                    // the next by 2, and so on; up to nine #'s are allowed
                    let tok = ctx.parser.gullet.pop_token().unwrap();
                    let txt = &tok.content;
                    // If it is not only a digit of 1-9
                    if !(txt.len() == 1 && txt.chars().next().unwrap().is_digit(10) && txt != "0") {
                        panic!();
                    }

                    let value = txt.parse::<u8>().unwrap();
                    if value as usize != num_args + 1 {
                        panic!();
                    }

                    num_args += 1;
                    delimiters.push(Vec::new());
                } else if tok.content == "EOF" {
                    panic!();
                } else {
                    delimiters[num_args].push(tok.content.into_owned().into());
                }
            }

            // replacement text, enclosed in '{' and '}' and properly nested
            let arg = ctx.parser.gullet.consume_arg::<&str>(&[]).unwrap();
            let mut tokens = arg.tokens;

            if let Some(insert) = insert {
                tokens.insert(0, insert);
            }

            let tokens = if ctx.func_name == "\\edef" || ctx.func_name == "\\xdef" {
                let mut tokens = ctx.parser.gullet.expand_tokens(tokens.into_iter()).unwrap();
                tokens.reverse();
                tokens
            } else {
                tokens
            };

            // This is annoying
            let tokens = tokens
                .into_iter()
                .map(Token::into_owned)
                .collect::<Vec<_>>();

            // FInal arg is the expansion of the macro
            ctx.parser.gullet.macros.set_back_macro(
                token.content.into_owned(),
                Some(Arc::new(MacroReplace::Expansion(MacroExpansion {
                    tokens,
                    num_args: num_args as u16,
                    delimiters: Some(delimiters),
                    unexpandable: false,
                }))),
            );

            ParseNode::Internal(InternalNode {
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        // TODO:
        #[cfg(feature = "html")]
        html_builder: None,
    });

    fns.insert_for_all_str(["\\def", "\\gdef", "\\edef", "\\xdef"].into_iter(), def);

    let global_let = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Internal, 0)
            .with_allowed_in_text(true)
            .with_primitive(true),
        handler: Box::new(|ctx, _, _| {
            let token = ctx.parser.gullet.pop_token().unwrap();
            let name = token.content;
            if CONTROL_SEQUENCE_REGEX.is_match(&name) {
                panic!();
            }

            ctx.parser.gullet.consume_spaces().unwrap();

            let mut tok = get_rhs(ctx.parser).unwrap();
            let global = ctx.func_name == "\\\\globallet";
            let_command(ctx.parser, name.into_owned(), &mut tok, global);

            ParseNode::Internal(InternalNode {
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        // TODO:
        #[cfg(feature = "html")]
        html_builder: None,
    });

    fns.insert_for_all_str(["\\let", "\\\\globallet"].into_iter(), global_let);

    let future = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Internal, 0)
            .with_allowed_in_text(true)
            .with_primitive(true),
        handler: Box::new(|ctx, _, _| {
            let token = ctx.parser.gullet.pop_token().unwrap();
            let name = token.content;
            if CONTROL_SEQUENCE_REGEX.is_match(&name) {
                panic!();
            }

            let middle = ctx.parser.gullet.pop_token().unwrap();

            let mut tok = ctx.parser.gullet.pop_token().unwrap();

            let global = ctx.func_name == "\\\\globalfuture";
            let_command(ctx.parser, name.into_owned(), &mut tok, global);

            ctx.parser.gullet.push_token(tok);
            ctx.parser.gullet.push_token(middle);

            ParseNode::Internal(InternalNode {
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        // TODO:
        #[cfg(feature = "html")]
        html_builder: None,
    });

    fns.insert_for_all_str(["\\futurelet", "\\\\globalfuture"].into_iter(), future);
}

const GLOBAL_MAP: &'static [(&'static str, &'static str)] = &[
    ("\\global", "\\global"),
    ("\\long", "\\\\globallong"),
    ("\\\\globallong", "\\\\globallong"),
    ("\\def", "\\gdef"),
    ("\\gdef", "\\gdef"),
    ("\\edef", "\\xdef"),
    ("\\xdef", "\\xdef"),
    ("\\let", "\\\\globallet"),
    ("\\futurelet", "\\\\globalfuture"),
];
fn get_global_map(text: &str) -> Option<&'static str> {
    GLOBAL_MAP.iter().find(|(l, _)| *l == text).map(|(_, r)| *r)
}

static CONTROL_SEQUENCE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("^(?:[\\{}$&#^_]|EOF)$").unwrap());

fn get_rhs<'a, 'f>(parser: &mut Parser<'a, 'f>) -> Result<Token<'a>, ParseError> {
    let mut tok = parser.gullet.pop_token()?;
    if tok.content == "=" {
        tok = parser.gullet.pop_token()?;
        if tok.content == " " {
            tok = parser.gullet.pop_token()?;
        }
    }

    Ok(tok)
}

fn let_command<'a, 'f>(
    parser: &mut Parser<'a, 'f>,
    name: String,
    tok: &mut Token<'a>,
    global: bool,
) {
    let macr = parser.gullet.macros.get_back_macro(&tok.content);

    // TODO: This and other usages of tokens make me wonder whether katex is (ab)using references
    // to tokens to cause them to share information?
    if let Some(macr) = macr {
        parser
            .gullet
            .macros
            .set_back_macro(name, Some(macr.clone()));
    } else {
        tok.no_expand = true;
        let is_expandable = parser.gullet.is_expandable(&tok.content);
        let macr = Some(Arc::new(MacroReplace::Expansion(MacroExpansion {
            tokens: vec![tok.clone().into_owned()],
            num_args: 0,
            delimiters: None,
            unexpandable: !is_expandable,
        })));

        if global {
            parser.gullet.macros.set_global_back_macro(name, macr);
        } else {
            parser.gullet.macros.set_back_macro(name, macr);
        }
    };
}
