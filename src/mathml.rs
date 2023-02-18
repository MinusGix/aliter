use std::any::{Any, TypeId};

use crate::{
    build_common::FONT_MAP,
    dom_tree::{CssStyle, Span},
    expander::Mode,
    font_metrics::get_character_metrics,
    functions,
    mathml_tree::{EmptyMathNode, MathNode, MathNodeType, MathmlNode, TextNode, WithMathDomNode},
    parse_node::{ParseNode, SymbolParseNode, TextOrdNode},
    symbols::{self, LIGATURES},
    tree::ClassList,
    util::{char_code_for, find_assoc_data, FontVariant},
    FontShape, FontWeight, Options,
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

/// Returns the math variant, or `None` if none is required.
pub(crate) fn get_variant<G: SymbolParseNode + Any + 'static>(
    group: &G,
    options: &Options,
) -> Option<FontVariant> {
    // Handle \text... font specifiers as best we can.
    // MathML has a limited list of allowable mathvariant specifiers; see
    // https://www.w3.org/TR/MathML3/chapter3.html#presm.commatt
    if options.font_family == "texttt" {
        return Some(FontVariant::Monospace);
    } else if options.font_family == "textsf" {
        if options.font_shape == Some(FontShape::TextIt)
            && options.font_weight == Some(FontWeight::TextBf)
        {
            return Some(FontVariant::SansSerifBoldItalic);
        } else if options.font_shape == Some(FontShape::TextIt) {
            return Some(FontVariant::SansSerifItalic);
        } else if options.font_weight == Some(FontWeight::TextBf) {
            return Some(FontVariant::BoldSansSerif);
        } else {
            return Some(FontVariant::SansSerif);
        }
    } else if options.font_shape == Some(FontShape::TextIt)
        && options.font_weight == Some(FontWeight::TextBf)
    {
        return Some(FontVariant::BoldItalic);
    } else if options.font_shape == Some(FontShape::TextIt) {
        return Some(FontVariant::Italic);
    } else if options.font_weight == Some(FontWeight::TextBf) {
        return Some(FontVariant::Bold);
    }

    let font = &options.font;

    if font.is_empty() || font == "mathnormal" {
        return None;
    }

    // let mode = group.mode;
    match font.as_ref() {
        "mathit" => return Some(FontVariant::Italic),
        "boldsymbol" => {
            if group.type_id() == TypeId::of::<TextOrdNode>() {
                return Some(FontVariant::Bold);
            } else {
                return Some(FontVariant::BoldItalic);
            }
        }
        "mathbf" => return Some(FontVariant::Bold),
        "mathbb" => return Some(FontVariant::DoubleStruck),
        "mathfrak" => return Some(FontVariant::Fraktur),
        // MathML makes no distinction between script and caligraphic
        "mathscr" | "mathcal" => return Some(FontVariant::Script),
        "mathsf" => return Some(FontVariant::SansSerif),
        "mathtt" => return Some(FontVariant::Monospace),
        _ => {}
    }

    let text = group.text();
    if text == "\\imath" || text == "\\jmath" {
        return None;
    }

    let mode = group.info().mode;

    let replace = symbols::SYMBOLS.get(mode, text).and_then(|s| s.replace);
    let text = replace.unwrap_or(text);
    let text_char = text.chars().nth(0).unwrap();

    let font_data = find_assoc_data(FONT_MAP, font).unwrap();
    if get_character_metrics(text_char, font_data.font, mode).is_some() {
        return Some(font_data.variant);
    }

    None
}

pub(crate) fn build_expression(
    expression: &[ParseNode],
    options: &Options,
    is_ord_group: Option<bool>,
) -> Vec<MathmlNode> {
    if expression.len() == 1 {
        let first = expression.iter().nth(0).unwrap();
        let mut group = build_group(Some(first), options);
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
    expression: &[ParseNode],
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

/// Takes a full parse tree and settings and builds a MathML representation of it. In particular,
/// we put the elements from building the parse tree into a `<semantics>` tag so we can also
/// include that TeX source as an annotation.
///
/// Note that we actually return a dom tree element with a `<math>` inside itso we can do
/// appropriate styling.
pub fn build_mathml(
    tree: &[ParseNode],
    tex_expr: &str,
    options: &Options,
    is_display_mode: bool,
    for_mathml_only: bool,
) -> Span<MathmlNode> {
    let expression = build_expression(tree, options, None);

    // TODO: Make a pass thru the MathML similar to buildHTML.traverseNonSpaceNodes
    // and add spacing nodes. This is necessary only adjacent to math operators
    // like \sin or \lim or to subsup elements that contain math operators.
    // MathML takes care of the other spacing issues.

    // Wrap up the expression in a row so it is presented in the semantics tag correctly, unless
    // it is a single `<mrow>` or `<mtable>`.
    let wrapper = if expression.len() == 1 {
        if let MathmlNode::Math(expr) = &expression[0] {
            if expr.typ == MathNodeType::MRow || expr.typ == MathNodeType::MTable {
                Some(expr.clone())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    let wrapper =
        wrapper.unwrap_or_else(|| MathNode::new(MathNodeType::MRow, expression, ClassList::new()));

    // Build a TeX annotation of the source
    let tex_text = TextNode::new(tex_expr.to_string());
    let annotation = MathNode::new(MathNodeType::Annotation, vec![tex_text], ClassList::new())
        .with_attribute("encoding", "application/x-tex");

    let semantics: MathNode<MathmlNode> = MathNode::new(
        MathNodeType::Semantics,
        vec![wrapper.into(), annotation.into()],
        ClassList::new(),
    );

    let mut math = MathNode::new(MathNodeType::Math, vec![semantics], ClassList::new())
        .with_attribute("xmlns", "http://www.w3.org/1998/Math/MathML");
    if is_display_mode {
        math.set_attribute("display", "block");
    }

    // You can't style `<math>` nodes, so we wrap the node in a span.
    let wrapper_class = if for_mathml_only {
        "katex"
    } else {
        "katex-mathml"
    };

    // We create the span directly rather than through make_span, like katex does.
    // make_span calls size_element_from_children but that just sets height/depth/max_font_size to
    // 0.0 if they don't actually exist on the object (like they don't on a math node).
    // and we don't actually need to bother with those.
    Span::new(
        vec![wrapper_class.to_string()],
        vec![math.into()],
        None,
        CssStyle::default(),
    )
}
