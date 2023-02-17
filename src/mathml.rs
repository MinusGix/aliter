use crate::{
    expander::Mode,
    functions,
    mathml_tree::{EmptyMathNode, MathNode, MathNodeType, TextNode, WithMathDomNode},
    parse_node::ParseNode,
    symbols::{self, ligatures},
    tree::ClassList,
    util::char_code_for,
    Options,
};

/// Takes a symbol and converts it into a MathML text node after performing optional replacement
/// /from symbols.rs
pub(crate) fn make_text(text: String, mode: Mode, options: Option<&Options>) -> TextNode {
    let text_char = text.chars().nth(0);
    let text_char_code = text_char.map(char_code_for);

    let replace = symbols::SYMBOLS.get(mode, &text).and_then(|s| s.replace);

    if let Some(replace) = replace {
        if text_char_code == Some(0xD835) && !ligatures.contains(&text.as_str()) {
            if let Some(options) = options {
                let font_family_tt =
                    options.font_family.len() > 4 && options.font_family[4..].starts_with("tt");
                let font_tt = options.font.len() > 4 && options.font[4..].starts_with("tt");
                if font_family_tt || font_tt {
                    return TextNode::new(replace.to_string());
                }
            }
        }
    }

    TextNode::new(text)
}

// TODO: this should be able to avoid boxing
/// Wrap the given array of notes in an `<mrow>` node if needed, i.e., unless the array has length
/// 1. Always returns a single node.
pub(crate) fn make_row(body: Vec<Box<dyn WithMathDomNode>>) -> Box<dyn WithMathDomNode> {
    if body.len() == 1 {
        body[0]
    } else {
        Box::new(MathNode::new(MathNodeType::MRow, body, ClassList::new()))
    }
}

pub(crate) fn build_group(
    group: Option<&ParseNode>,
    options: &Options,
) -> Box<dyn WithMathDomNode> {
    let Some(group) = group else {
        return Box::new(MathNode::<EmptyMathNode>::new_empty(MathNodeType::MRow));
    };

    if let Some(mathml_builder) = functions::FUNCTIONS.find_mathml_builder_for_type(group.typ()) {
        mathml_builder(group, options)
    } else {
        panic!("Got group of unknown type")
    }
}
