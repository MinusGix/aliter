use std::{borrow::Cow, collections::HashMap};

use crate::{
    dom_tree::DomSpan, html::build_html, parse_node::ParseNode, util, Options, ParserConfig,
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct EmptyNode;
impl VirtualNode for EmptyNode {
    fn to_markup(&self) -> String {
        String::new()
    }
}

#[cfg(feature = "html")]
fn display_wrap(node: DomSpan, conf: ParserConfig) -> DomSpan {
    use crate::{build_common::make_span, dom_tree::CssStyle};

    if conf.display_mode {
        let mut classes = vec!["katex-display".to_string()];
        if conf.leq_no {
            classes.push("leqno".to_string());
        }

        if conf.fleqn {
            classes.push("fleqn".to_string());
        }

        make_span(classes, vec![node], None, CssStyle::default()).into_dom_span()
    } else {
        node
    }
}

#[cfg(feature = "html")]
fn build_html_tree(tree: Vec<ParseNode>, expr: &str, conf: ParserConfig) -> DomSpan {
    use crate::{build_common::make_span, dom_tree::CssStyle};

    let options = Options::from_parser_conf(&conf);

    let html_node = build_html(tree, options);
    // let katex_node = make_span(
    //     vec!["katex".to_string()],
    //     vec![html_node],
    //     None,
    //     CssStyle::default(),
    // );

    // display_wrap(katex_node, conf)
    todo!()
}
