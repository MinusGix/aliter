use std::{borrow::Cow, collections::HashMap};

use crate::tree::VirtualNode;

// TODO: We could have so fields that are numbers have to be actual numbers?
#[derive(Debug, Clone, Default)]
pub struct CssStyle {
    pub background_color: Option<Cow<'static, str>>,
    pub border_bottom_width: Option<Cow<'static, str>>,
    pub border_color: Option<Cow<'static, str>>,
    pub border_right_style: Option<Cow<'static, str>>,
    pub border_right_width: Option<Cow<'static, str>>,
    pub border_top_width: Option<Cow<'static, str>>,
    pub border_style: Option<Cow<'static, str>>,
    pub border_width: Option<Cow<'static, str>>,
    pub bottom: Option<Cow<'static, str>>,
    pub color: Option<Cow<'static, str>>,
    pub height: Option<Cow<'static, str>>,
    pub left: Option<Cow<'static, str>>,
    pub margin: Option<Cow<'static, str>>,
    pub margin_left: Option<Cow<'static, str>>,
    pub margin_right: Option<Cow<'static, str>>,
    pub margin_top: Option<Cow<'static, str>>,
    pub min_width: Option<Cow<'static, str>>,
    pub padding_left: Option<Cow<'static, str>>,
    pub position: Option<Cow<'static, str>>,
    pub top: Option<Cow<'static, str>>,
    pub width: Option<Cow<'static, str>>,
    pub vertical_align: Option<Cow<'static, str>>,
}

#[derive(Debug, Clone)]
pub struct HtmlDomNode {
    // TODO: Vec of enum?
    pub classes: Vec<String>,
    pub height: f64,
    pub depth: f64,
    pub max_font_size: f64,
    pub style: CssStyle,
}
impl HtmlDomNode {
    pub fn has_class(&self, class: &str) -> bool {
        self.classes.iter().any(|x| x == class)
    }
}
impl Default for HtmlDomNode {
    fn default() -> Self {
        Self {
            classes: Vec::new(),
            height: 0.0,
            depth: 0.0,
            max_font_size: 0.0,
            style: CssStyle::default(),
        }
    }
}
impl VirtualNode for HtmlDomNode {
    fn to_markup(&self) -> String {
        todo!()
    }
}

pub struct Span<T: VirtualNode> {
    pub node: HtmlDomNode,
    pub children: Vec<T>,
    pub attributes: HashMap<String, String>,
    pub width: Option<f64>,
}
impl<T: VirtualNode> Span<T> {}

pub struct Anchor {
    pub node: HtmlDomNode,
    pub children: Vec<HtmlDomNode>,
}

pub struct Img {
    pub node: HtmlDomNode,
    pub src: String,
    pub alt: String,
}

/// A symbol node contains information about a single symbol.  
/// It either renders to a single text node, or a span with a single text node in it,
/// depending on whether has CSS classes, styles, or needs italic correction.
pub struct SymbolNode {
    pub node: HtmlDomNode,
    pub text: String,
    // TODO: is this a flaot?
    pub italic: usize,
    pub skew: f64,
}

// pub struct SvgNode {
//     pub children: Vec<SvgChildNode>,
//     pub attributes: HashMap<String, String>,
// }

pub struct PathNode {
    pub path_name: String,
    pub alternate: Option<String>,
}

pub struct LineNode {
    pub attributes: HashMap<String, String>,
}
