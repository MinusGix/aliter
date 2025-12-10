use std::sync::Arc;

use crate::parse_node::{ParseNode, ParseNodeType};

use super::{BuilderFunctionSpec, FunctionPropSpec, Functions};

#[cfg(feature = "html")]
use crate::{build_common, dom_tree::CssStyle, Mode};
#[cfg(feature = "mathml")]
use crate::mathml_tree::{MathNode, MathNodeType, TextNode};
#[cfg(any(feature = "html", feature = "mathml"))]
use crate::tree::ClassList;

/// Converts verb group into body string.
///
/// \verb* replaces each space with an open box \u2423
/// \verb replaces each space with a no-break space \xA0
fn make_verb(body: &str, star: bool) -> String {
    if star {
        body.replace(' ', "\u{2423}")
    } else {
        body.replace(' ', "\u{00A0}")
    }
}

pub fn add_functions(fns: &mut Functions) {
    // \verb is handled in the parser/lexer, so we just need the builder
    let verb_builder = Arc::new(BuilderFunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Verb, 0),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            let ParseNode::Verb(verb) = group else { panic!() };

            let text = make_verb(&verb.body, verb.star);
            let mut body: Vec<crate::dom_tree::HtmlNode> = Vec::new();

            // \verb enters text mode and therefore is sized like \textstyle
            let new_options = options.having_style(options.style.text());
            let new_options = new_options.as_ref().unwrap_or(options);

            for c in text.chars() {
                let ch_str = if c == '~' {
                    "\\textasciitilde".to_string()
                } else {
                    c.to_string()
                };
                // Build each character as a symbol in Typewriter font
                let sym = build_common::make_symbol(
                    &ch_str,
                    "Typewriter-Regular",
                    Mode::Text,
                    Some(new_options),
                    vec!["mord".to_string(), "texttt".to_string()],
                );
                body.push(sym.into());
            }

            let mut classes = vec!["mord".to_string(), "text".to_string()];
            classes.extend(new_options.sizing_classes(options));

            build_common::make_span(classes, body, Some(new_options), CssStyle::default()).into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, _options| {
            let ParseNode::Verb(verb) = group else { panic!() };

            let text = make_verb(&verb.body, verb.star);
            let text_node = TextNode::new(text);
            let mut node = MathNode::<crate::mathml_tree::MathmlNode>::new(
                MathNodeType::MText,
                vec![text_node.into()],
                ClassList::new(),
            );
            node.set_attribute("mathvariant", "monospace");
            node.into()
        })),
    });
    fns.insert_builder(verb_builder);
}
