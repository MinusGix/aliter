#![allow(clippy::upper_case_acronyms)]

use std::borrow::Cow;

use expander::Mode;
use lexer::Token;
use parse_node::{Color, NodeInfo, ParseNode, TagNode};
use parser::{ParseError, Parser, ParserConfig};
use style::StyleId;

pub mod array;
pub mod build_common;
mod builtin_macros;
pub mod dom_tree;
pub mod environments;
pub mod expander;
pub mod functions;
#[cfg(feature = "html")]
pub mod html;
pub mod lexer;
pub mod macr;
pub mod namespace;
pub mod parse_node;
pub mod parser;
pub mod style;
pub mod symbols;
mod thing;
pub mod tree;
pub mod unicode;
pub mod unit;
mod util;

// TODO: expose our 'KaTeX' version?

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    TextBf,
    TextMd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontShape {
    TextIt,
    TextUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontMetrics {
    // TODO: is this a float?
    css_em_per_mu: usize,
}

pub struct Options {
    pub style: StyleId,
    pub color: Option<Color>,
    // TODO: is this a float?
    pub size: usize,
    // TODO: is this a float?
    pub text_size: usize,
    pub phantom: bool,
    pub font: Cow<'static, str>,
    pub font_family: Cow<'static, str>,
    pub font_weight: Option<FontWeight>,
    pub font_shape: Option<FontShape>,
    // TODO: is this a float?
    pub size_multiplier: usize,
    // TODO: is this a float?
    pub max_size: usize,
    // TODO: is this a float?
    pub min_rule_thickness: usize,
    pub font_metrics: FontMetrics,
}

/// For now this is simply exported
pub fn parse_tree<'a>(input: &'a str, conf: ParserConfig) -> Result<Vec<ParseNode>, ParseError> {
    let display_mode = conf.display_mode;
    let functions = &functions::FUNCTIONS;
    let mut parser = Parser::new(input, conf, functions);

    // TODO: Do we actually need to do these deletes as we don't currently use the same
    // macros structures each time?
    parser.gullet.macros.current.take_back_macro("\\df@tag");

    let tree = parser.dispatch_parse()?;

    parser
        .gullet
        .macros
        .current
        .take_back_macro("\\current@color");
    parser.gullet.macros.current.take_back_macro("\\color");

    if parser.gullet.macros.contains_back_macro("\\df@tag") {
        if !display_mode {
            return Err(ParseError::TagOnlyDisplayEquation);
        }

        Ok(vec![ParseNode::Tag(TagNode {
            body: tree,
            tag: parser.sub_parse(std::iter::once(Token::new_text("\\df@tag")))?,
            info: NodeInfo::new_mode(Mode::Text),
        })])
    } else {
        Ok(tree)
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse_tree, parser::ParserConfig};

    #[test]
    fn test_parse_tree() {
        let conf = ParserConfig::default();

        // TODO: This is just testing that they output values and not errors, but
        // we could do value tests for these.
        let simple_num = parse_tree("4", conf.clone()).unwrap();
        dbg!(simple_num);
        let basic_num = parse_tree("52", conf.clone()).unwrap();
        dbg!(basic_num);

        let basic_expr = parse_tree("9 + 12", conf.clone()).unwrap();
        dbg!(basic_expr);

        let paren_expr = parse_tree("(42 + 9)", conf.clone()).unwrap();
        dbg!(paren_expr);

        let basic_block = parse_tree("{4}", conf.clone()).unwrap();
        dbg!(basic_block);

        let basic_frac = parse_tree(r#"\frac{3}{9}"#, conf.clone()).unwrap();
        dbg!(basic_frac);
    }
}
