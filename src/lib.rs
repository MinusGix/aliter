#![allow(clippy::upper_case_acronyms)]

use std::borrow::Cow;

use expander::Mode;
use font_metrics::{get_global_metrics, FontMetrics};
use lexer::Token;
use once_cell::sync::OnceCell;
use parse_node::{Color, NodeInfo, ParseNode, TagNode};
use parser::{ParseError, Parser, ParserConfig};
use style::{StyleId, DISPLAY_STYLE, TEXT_STYLE};
use tree::ClassList;
use unit::Em;

pub mod array;
pub mod build_common;
mod builtin_macros;
pub mod delimiter;
#[cfg(feature = "html")]
pub mod dom_tree;
pub mod environments;
pub mod expander;
pub mod font_metrics;
mod font_metrics_data;
pub mod functions;
#[cfg(feature = "html")]
pub mod html;
pub mod lexer;
pub mod macr;
pub mod mathml;
#[cfg(feature = "mathml")]
pub mod mathml_tree;
pub mod namespace;
pub mod parse_node;
pub mod parser;
mod spacing_data;
pub mod style;
pub mod symbols;
pub mod tree;
pub mod unicode;
pub mod unicode_scripts;
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

const SIZE_STYLE_MAP: [[u8; 3]; 11] = [
    // Each element contains [textsize, scriptsize, scriptscriptsize].
    // The size mappings are taken from TeX with \normalsize=10pt.
    [1, 1, 1],   // size1: [5, 5, 5]              \tiny
    [2, 1, 1],   // size2: [6, 5, 5]
    [3, 1, 1],   // size3: [7, 5, 5]              \scriptsize
    [4, 2, 1],   // size4: [8, 6, 5]              \footnotesize
    [5, 2, 1],   // size5: [9, 6, 5]              \small
    [6, 3, 1],   // size6: [10, 7, 5]             \normalsize
    [7, 4, 2],   // size7: [12, 8, 6]             \large
    [8, 6, 3],   // size8: [14.4, 10, 7]          \Large
    [9, 7, 6],   // size9: [17.28, 12, 10]        \LARGE
    [10, 8, 7],  // size10: [20.74, 14.4, 12]     \huge
    [11, 10, 9], // size11: [24.88, 20.74, 17.28] \HUGE
];

const SIZE_MULTIPLIERS: [f64; 11] = [
    // fontMetrics.js:getGlobalMetrics also uses size indexes, so if
    // you change size indexes, change that function.
    0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.2, 1.44, 1.728, 2.074, 2.488,
];

fn size_at_style(size: usize, style: StyleId) -> usize {
    if style.size() < 2 {
        size
    } else {
        SIZE_STYLE_MAP[size - 1][style.size() - 1] as usize
    }
}

const BASE_SIZE: usize = 6;
#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    pub style: StyleId,
    pub color: Option<Color>,
    // TODO: is this a float? It is used to index an array..
    pub size: usize,
    pub text_size: usize,
    pub phantom: bool,
    pub font: Cow<'static, str>,
    pub font_family: Cow<'static, str>,
    pub font_weight: Option<FontWeight>,
    pub font_shape: Option<FontShape>,
    pub max_size: Em,
    pub min_rule_thickness: Em,
    /// This is separate per options instance
    font_metrics: OnceCell<FontMetrics>,
}
impl Options {
    pub(crate) fn from_parser_conf(conf: &ParserConfig) -> Self {
        Options {
            style: if conf.display_mode {
                DISPLAY_STYLE
            } else {
                TEXT_STYLE
            },
            color: None,
            size: BASE_SIZE,
            text_size: BASE_SIZE,
            phantom: false,
            font: Cow::Borrowed(""),
            font_family: Cow::Borrowed(""),
            font_weight: None,
            font_shape: None,
            max_size: conf.max_size,
            min_rule_thickness: conf.min_rule_thickness,
            font_metrics: OnceCell::new(),
        }
    }

    // TODO: Can we just do this on normal `Clone`?
    /// The intended method of cloning the options instance and then altering it. This clears the
    /// font metrics cache.
    pub fn clone_alter(&self) -> Options {
        let mut opts = self.clone();
        opts.font_metrics = Default::default();
        opts
    }

    pub fn size_multiplier(&self) -> f64 {
        SIZE_MULTIPLIERS[self.size - 1]
    }

    // TODO: Should we rename the 'having' methods to be more in line with Rust's naming conventions?
    // TODO: Should we just have separate versions that actually clone to completely mimic the katex api?

    /// Returns an options object with the given style  
    /// Returns `None` if there was no changes needed
    pub fn having_style(&self, style: StyleId) -> Option<Options> {
        if self.style == style {
            None
        } else {
            let mut opts = self.clone_alter();
            opts.style = style;
            opts.size = size_at_style(self.text_size, style);
            Some(opts)
        }
    }

    /// Returns an options object with a cramped version of the current style.  
    /// Returns `None` if there was no changes needed.
    pub fn having_cramped_style(&self) -> Option<Options> {
        self.having_style(self.style.cramp())
    }

    /// Returns an options object with the given size and in at least `\textstyle`.  
    /// Returns `None` if there was no changes needed.
    pub fn having_size(&self, size: usize) -> Option<Options> {
        if self.size == size && self.text_size == size {
            None
        } else {
            let mut opts = self.clone_alter();
            opts.style = self.style.text();
            opts.size = size;
            opts.text_size = size;
            Some(opts)
        }
    }

    /// Like applying `having_size(BASE_SIZE)` and then `having_style(style)`.  
    /// If there is no `style`, then it changes to at least `\textstyle`  
    /// Returns `None` if there was no changes needed.
    pub fn having_base_style(&self, style: Option<StyleId>) -> Option<Options> {
        let style = style.unwrap_or_else(|| self.style.text());
        let want_size = size_at_style(BASE_SIZE, style);
        if self.size == want_size && self.text_size == BASE_SIZE && self.style == style {
            None
        } else {
            let mut opts = self.clone_alter();
            opts.style = style;
            opts.size = want_size;
            Some(opts)
        }
    }

    /// Remove the effect of sizing changes such as `\Huge`.  
    /// Keep the effect of the current style, such as `\scriptstyle`.  
    pub fn having_base_sizing(&self) -> Options {
        let size = match self.style.as_id() {
            // normalsize in scriptstyle
            4 | 5 => 3,
            // normalsize in scriptscriptstyle
            6 | 7 => 1,
            // normalsize in textstyle or displaystyle
            _ => 6,
        };

        let mut opts = self.clone_alter();
        opts.style = self.style.text();
        opts.size = size;

        opts
    }

    // These with functions are more in line with typical Rust conventions
    // TODO: Do we really need to clear the font metrics cache on each of these? They aren't even
    // used for it in the typical initialization case? Check that it isn't initialized anywhere else.

    pub fn with_color(mut self, color: Color) -> Options {
        self.color = Some(color);
        self.font_metrics = Default::default();
        self
    }

    pub fn with_phantom(mut self) -> Options {
        self.phantom = true;
        self.font_metrics = Default::default();
        self
    }

    pub fn with_font(mut self, font: impl Into<Cow<'static, str>>) -> Options {
        self.font = font.into();
        self.font_metrics = Default::default();
        self
    }

    pub fn with_text_font_family(mut self, font_family: impl Into<Cow<'static, str>>) -> Options {
        self.font_family = font_family.into();
        self.font_metrics = Default::default();
        self
    }

    pub fn with_font_weight(mut self, font_weight: FontWeight) -> Options {
        self.font_weight = Some(font_weight);
        self.font_metrics = Default::default();
        self
    }

    pub fn with_text_font_shape(mut self, font_shape: FontShape) -> Options {
        self.font_shape = Some(font_shape);
        self.font_metrics = Default::default();
        self
    }

    /// Returns the CSS sizing classes required to switch from enclosing options `oldOptions` to
    /// `self`.
    pub fn sizing_classes(&self, old_options: &Options) -> ClassList {
        if old_options.size != self.size {
            vec![
                "sizing".to_string(),
                format!("reset-size{}", old_options.size),
                format!("size{}", self.size),
            ]
        } else {
            ClassList::new()
        }
    }

    /// Return the CSS sizing classes required to switch to the base size.
    pub fn base_sizing_classes(&self) -> ClassList {
        if self.size != BASE_SIZE {
            vec![
                "sizing".to_string(),
                format!("reset-size{}", self.size),
                format!("size{}", BASE_SIZE),
            ]
        } else {
            ClassList::new()
        }
    }

    /// Get the font metrics, initializing if needed.
    pub fn font_metrics(&self) -> &FontMetrics {
        let size = self.size;
        self.font_metrics.get_or_init(|| get_global_metrics(size))
    }

    pub fn get_color(&self) -> Option<Color> {
        if self.phantom {
            Some(Color::Named(Cow::Borrowed("transparent")))
        } else {
            self.color.clone()
        }
    }
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

#[cfg(any(feature = "html", feature = "mathml"))]
pub fn render_error(
    err: ParseError,
    expr: &str,
    conf: ParserConfig,
) -> dom_tree::Span<dom_tree::SymbolNode> {
    use dom_tree::{CssStyle, SymbolNode};

    // TODO: options.throwOnError?

    let symbol = SymbolNode::new_text(expr.to_string());
    let mut node = build_common::make_span(
        vec!["katex-error".to_string()],
        vec![symbol],
        None,
        CssStyle::default(),
    );
    // TODO: we probably want a separate impl
    node.attributes
        .insert("title".to_string(), format!("{:?}", err));
    node.attributes.insert(
        "style".to_string(),
        format!("color:{}", conf.error_color.to_string()),
    );

    node
}

#[cfg(feature = "html")]
pub fn render_to_html_tree(expr: &str, conf: ParserConfig) -> dom_tree::DomSpan {
    todo!()
    // match parse_tree(expr, conf.clone()) {
    //     Ok(tree) => build_html_tree(tree, expr, conf),
    //     Err(err) => render_error(err, expr, conf).into_dom_span(),
    // }
}

// TODO: websys render function

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
