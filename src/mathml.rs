use crate::{
    expander::Mode,
    functions,
    mathml_tree::{EmptyMathNode, MathNode, MathNodeType, MathmlNode, TextNode, WithMathDomNode},
    parse_node::ParseNode,
    symbols::{self, LIGATURES},
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
        if text_char_code == Some(0xD835) && !LIGATURES.contains(&text.as_str()) {
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

/// Wrap the given array of notes in an `<mrow>` node if needed, i.e., unless the array has length
/// 1. Always returns a single node.
pub(crate) fn make_row<T: WithMathDomNode + 'static>(body: Vec<T>) -> MathmlNode
where
    MathmlNode: From<T>,
{
    if body.len() == 1 {
        let val = body.into_iter().nth(0).unwrap();
        val.into()
    } else {
        MathNode::new(MathNodeType::MRow, body, ClassList::new()).into()
    }
}

// TODO: get_variant

pub(crate) fn build_expression(
    expression: Vec<ParseNode>,
    options: &Options,
    is_ord_group: Option<bool>,
) -> Vec<MathmlNode> {
    if expression.len() == 1 {
        let first = expression.into_iter().nth(0).unwrap();
        let mut group = build_group(Some(&first), options);
        if is_ord_group == Some(true) {
            if let MathmlNode::Math(group) = &mut group {
                if group.typ == MathNodeType::Mo {
                    // When TeX writers want to suppress spacing on an operator, they often
                    // put the operator by itself inside braces.
                    group.set_attribute("lspace", "0em");
                    group.set_attribute("rspace", "0em");
                }
            }
        }

        return vec![group];
    }

    let mut groups = Vec::new();
    let mut last_group_idx = None;
    for expr in expression {
        let mut group = build_group(Some(&expr), options);
        if let MathmlNode::Math(group) = &mut group {
            let last_group = last_group_idx.and_then(|idx| groups.get_mut(idx));
            if let Some(MathmlNode::Math(last_group)) = last_group {
                if group.typ == MathNodeType::MText
                    && last_group.typ == MathNodeType::MText
                    && group.get_attribute("mathvariant") == last_group.get_attribute("mathvariant")
                {
                    // Concatenate adjacent `<mtext`s
                    last_group.children.append(&mut group.children);
                    continue;
                } else if group.typ == MathNodeType::Mn && last_group.typ == MathNodeType::Mn {
                    // Concatenate adjacent `<mn>`s
                    last_group.children.append(&mut group.children);
                    continue;
                } else if group.typ == MathNodeType::Mi
                    && group.children.len() == 1
                    && last_group.typ == MathNodeType::Mn
                {
                    // Concatenate `<mn>...</mn>` followed by `<mi>.</mi>`
                    let child = group.children.get(0);
                    if let Some(MathmlNode::Text(child)) = child {
                        if child.text == "." {
                            last_group.children.append(&mut group.children);
                            continue;
                        }
                    }
                } else if last_group.typ == MathNodeType::Mi && last_group.children.len() == 1 {
                    let last_child = last_group.children.get(0);
                    if let Some(MathmlNode::Text(last_child)) = last_child {
                        if last_child.text == "\u{0338}"
                            && matches!(
                                group.typ,
                                MathNodeType::Mo | MathNodeType::Mi | MathNodeType::Mn
                            )
                        {
                            let child = group.children.get_mut(0);
                            if let Some(MathmlNode::Text(child)) = child {
                                if !child.text.is_empty() {
                                    // Overlay with combining character long solidus
                                    let first_char = child.text.chars().nth(0).unwrap();
                                    child.text = format!(
                                        "{}\u{0338}{}",
                                        first_char,
                                        &child.text[first_char.len_utf8()..]
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        groups.push(group);
        last_group_idx = Some(groups.len() - 1)
    }

    groups
}

/// Equivalent to [`build_expression`], but wraps the elements in an `<mrow>` if there's more than
/// one.
pub(crate) fn build_expression_row(
    expression: Vec<ParseNode>,
    options: &Options,
    is_ord_group: Option<bool>,
) -> MathmlNode {
    let res = build_expression(expression, options, is_ord_group);
    make_row(res)
}

pub(crate) fn build_group(group: Option<&ParseNode>, options: &Options) -> MathmlNode {
    let Some(group) = group else {
        return MathNode::<EmptyMathNode>::new_empty(MathNodeType::MRow).into();
    };

    if let Some(mathml_builder) = functions::FUNCTIONS.find_mathml_builder_for_type(group.typ()) {
        mathml_builder(group, options)
    } else {
        panic!("Got group of unknown type")
    }
}
