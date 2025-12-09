use std::{borrow::Cow, collections::HashMap};

use crate::{
    dom_tree::{HtmlDomNode, HtmlNode, Span, WithHtmlDomNode},
    html::build_html,
    mathml_tree::MathmlNode,
    parse_node::ParseNode,
    util, Options, ParserConfig,
};

// TODO: Vec of enum for common kinds?
pub type ClassList = Vec<String>;

// TODO: We could do better by having keys be Cow<'static, str>?
// Though I think you need a crate for a nicely behaving map type for that
pub type Attributes = HashMap<String, String>;

/// Returns the value that should go in `class="{}"`
pub(crate) fn class_attr(classes: &ClassList) -> Option<String> {
    if classes.is_empty() {
        None
    } else {
        // TODO: use intersperse instead
        Some(
            classes
                .iter()
                .filter(|c| !c.is_empty())
                .map(|class| util::escape(class.as_str()))
                .collect::<Vec<Cow<'_, str>>>()
                .join(" "),
        )
    }
}

pub trait VirtualNode {
    // TODO: We somehow need to support translating into HTML nodes
    // We could use websys?
    // fn into_node(self) -> Node;
    fn to_markup(&self) -> String;
}
impl<T: VirtualNode + ?Sized> VirtualNode for Box<T> {
    fn to_markup(&self) -> String {
        (**self).to_markup()
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct EmptyNode {
    node: HtmlDomNode,
}
impl VirtualNode for EmptyNode {
    fn to_markup(&self) -> String {
        String::new()
    }
}
impl WithHtmlDomNode for EmptyNode {
    fn node(&self) -> &HtmlDomNode {
        &self.node
    }

    fn node_mut(&mut self) -> &mut HtmlDomNode {
        &mut self.node
    }
}

#[cfg(feature = "html")]
fn display_wrap<T: WithHtmlDomNode>(node: Span<T>, conf: ParserConfig) -> Span<HtmlNode>
where
    HtmlNode: From<T>,
{
    use crate::{build_common::make_span, dom_tree::CssStyle};

    if conf.display_mode {
        let mut classes = vec!["katex-display".to_string()];
        if conf.leq_no {
            classes.push("leqno".to_string());
        }

        if conf.fleqn {
            classes.push("fleqn".to_string());
        }

        let span = make_span(classes, vec![node], None, CssStyle::default());
        span.using_html_node()
    } else {
        node.using_html_node()
    }
}

#[cfg(feature = "html")]
pub(crate) fn build_html_tree(tree: &[ParseNode], conf: ParserConfig) -> Span<HtmlNode> {
    use crate::{build_common::make_span, dom_tree::CssStyle};

    let options = Options::from_parser_conf(&conf);

    let html_node = build_html(tree, &options);
    let katex_node = make_span(
        vec!["katex".to_string()],
        vec![html_node],
        None,
        CssStyle::default(),
    );

    display_wrap(katex_node, conf)
}

#[cfg(feature = "mathml")]
pub(crate) fn build_mathml_tree(
    tree: &[ParseNode],
    expr: &str,
    conf: ParserConfig,
) -> Span<MathmlNode> {
    use crate::mathml::build_mathml;

    let options = Options::from_parser_conf(&conf);

    build_mathml(tree, expr, &options, conf.display_mode, true)
}

/// Multi-purpose node that can be either HTML or MathML
#[cfg(any(feature = "html", feature = "mathml"))]
pub enum MlNode {
    #[cfg(feature = "html")]
    Html(HtmlNode),
    #[cfg(feature = "mathml")]
    Mathml(MathmlNode),
    Span(Span<MlNode>),
}
#[cfg(any(feature = "html", feature = "mathml"))]
impl VirtualNode for MlNode {
    fn to_markup(&self) -> String {
        match self {
            #[cfg(feature = "html")]
            MlNode::Html(node) => node.to_markup(),
            #[cfg(feature = "mathml")]
            MlNode::Mathml(node) => node.to_markup(),
            MlNode::Span(node) => node.to_markup(),
        }
    }
}
#[cfg(feature = "html")]
impl From<HtmlNode> for MlNode {
    fn from(node: HtmlNode) -> Self {
        MlNode::Html(node)
    }
}
#[cfg(feature = "mathml")]
impl From<MathmlNode> for MlNode {
    fn from(node: MathmlNode) -> Self {
        MlNode::Mathml(node)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputType {
    #[cfg(feature = "html")]
    Html,
    #[cfg(feature = "mathml")]
    Mathml,
    #[cfg(all(feature = "html", feature = "mathml"))]
    HtmlAndMathml,
}

#[cfg(all(feature = "html", feature = "mathml"))]
#[allow(dead_code)]
pub(crate) fn build_tree(
    tree: &[ParseNode],
    expr: &str,
    conf: ParserConfig,
    output: OutputType,
) -> Span<MlNode> {
    use crate::{build_common, dom_tree::CssStyle, mathml::build_mathml};

    let options = Options::from_parser_conf(&conf);

    match output {
        #[cfg(feature = "html")]
        OutputType::Html => {
            let html = build_html(tree, &options);
            let node = build_common::make_span(
                vec!["katex".to_string()],
                vec![html],
                None,
                CssStyle::default(),
            );
            display_wrap(node, conf).map(MlNode::from)
        }
        #[cfg(feature = "mathml")]
        OutputType::Mathml => {
            build_mathml(tree, expr, &options, conf.display_mode, true).map(MlNode::from)
        }
        #[cfg(all(feature = "html", feature = "mathml"))]
        OutputType::HtmlAndMathml => {
            let html = build_html(tree, &options).map(MlNode::from);
            let height = html.node.height;
            let depth = html.node.depth;
            let max_font_size = html.node.max_font_size;
            let mathml =
                build_mathml(tree, expr, &options, conf.display_mode, false).map(MlNode::from);
            let mut node = Span::new(
                vec!["katex".to_string()],
                vec![html, mathml],
                None,
                CssStyle::default(),
            )
            .map(MlNode::Span);

            node.node.height = height;
            node.node.depth = depth;
            node.node.max_font_size = max_font_size;

            node
        }
    }
}
