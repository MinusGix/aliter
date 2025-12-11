//! Convert HtmlNode trees to IR representation.
//!
//! This module provides conversion from the HTML DOM tree to the IR format,
//! which proves the IR can represent everything the HTML builder produces.

use super::types::*;
use crate::dom_tree::{HtmlNode, WithHtmlDomNode};
use crate::unit::parse_em;

/// Convert an HtmlNode tree to IR MathElement.
pub fn convert(node: &HtmlNode) -> MathElement {
    convert_node(node)
}

fn convert_node(node: &HtmlNode) -> MathElement {
    match node {
        HtmlNode::Empty(_) => MathElement::Kern { width: 0.0 },

        HtmlNode::Symbol(sym) => {
            let style = TextStyle {
                color: sym.node.style.color.clone(),
                italic_correction: sym.italic,
                skew: sym.skew,
                size: sym.node.max_font_size.max(1.0),
                font: infer_font_from_classes(&sym.node.classes),
                width: Some(sym.width),
            };
            MathElement::Text {
                text: sym.text.clone(),
                style,
            }
        }

        HtmlNode::Span(span) => {
            let classes = &span.node.classes;

            // Check for special span types
            if classes.iter().any(|c| c == "mspace") {
                // Spacing element - extract width from margin-right or width style
                let width = span
                    .node
                    .style
                    .margin_right
                    .as_ref()
                    .and_then(|s| parse_em(s))
                    .or_else(|| span.node.style.width.as_ref().and_then(|s| parse_em(s)))
                    .unwrap_or(0.0);
                return MathElement::Kern { width };
            }

            if classes.iter().any(|c| c == "rule" || c == "hline" || c == "hdashline") {
                // Rule element
                let width = span.node.style.width.as_ref().and_then(|s| parse_em(s)).unwrap_or(0.0);
                let height = span.node.style.border_bottom_width.as_ref()
                    .and_then(|s| parse_em(s))
                    .unwrap_or(span.node.height);
                let style = if classes.iter().any(|c| c == "hdashline") {
                    LineStyle::Dashed
                } else {
                    LineStyle::Solid
                };
                return MathElement::Rule {
                    width,
                    height,
                    shift: 0.0,
                    style,
                    color: span.node.style.color.clone(),
                };
            }

            if classes.iter().any(|c| c == "nulldelimiter") {
                return MathElement::Kern { width: 0.0 };
            }

            // Check for vlist (vertical layout)
            if classes.iter().any(|c| c == "vlist-t" || c == "vlist") {
                return convert_vlist(span);
            }

            // Regular span - convert as HBox
            let children = convert_children(&span.children);
            MathElement::HBox {
                children,
                width: span.width.unwrap_or(0.0),
                height: span.node.height,
                depth: span.node.depth,
                classes: classes.clone(),
            }
        }

        HtmlNode::DocumentFragment(frag) => {
            let children = convert_children(&frag.children);
            MathElement::HBox {
                children,
                width: 0.0, // Will be computed from children
                height: frag.node.height,
                depth: frag.node.depth,
                classes: vec![],
            }
        }

        HtmlNode::Anchor(anchor) => {
            let inner = MathElement::HBox {
                children: convert_children(&anchor.children),
                width: 0.0,
                height: anchor.node.height,
                depth: anchor.node.depth,
                classes: vec![],
            };
            let href = anchor.attributes.get("href").cloned().unwrap_or_default();
            MathElement::Link {
                href,
                inner: Box::new(inner),
            }
        }

        HtmlNode::Img(img) => {
            let width = img.node.style.width.as_ref().and_then(|s| parse_em(s)).unwrap_or(0.0);
            let height = img.node.style.height.as_ref().and_then(|s| parse_em(s)).unwrap_or(0.0);
            MathElement::Image {
                src: img.src.clone(),
                alt: img.alt.clone(),
                width,
                height,
            }
        }

        HtmlNode::Svg(svg) => {
            // Extract path data from SVG children
            let path_data = svg
                .children
                .iter()
                .filter_map(|child| {
                    if let crate::dom_tree::SvgChildNode::Path(path) = child {
                        Some(path.path_name.clone())
                    } else {
                        None
                    }
                })
                .next()
                .unwrap_or(std::borrow::Cow::Borrowed(""));

            let width = svg.attributes.get("width").and_then(|s| parse_em(s)).unwrap_or(0.0);
            let height = svg.attributes.get("height").and_then(|s| parse_em(s)).unwrap_or(0.0);

            MathElement::Path {
                path_data,
                width,
                height,
                shift: 0.0,
            }
        }
    }
}

fn convert_children(children: &[HtmlNode]) -> Vec<Positioned<MathElement>> {
    let mut result = Vec::new();
    let mut x_offset = 0.0;

    for child in children {
        let element = convert_node(child);
        let width = element.width();

        // Check for explicit positioning via style
        let node = child.node();
        let y_offset = node
            .style
            .vertical_align
            .as_ref()
            .and_then(|s| parse_em(s))
            .unwrap_or(0.0);

        result.push(Positioned::new(element, x_offset, y_offset));
        x_offset += width;
    }

    result
}

fn convert_vlist<T: crate::tree::VirtualNode>(span: &crate::dom_tree::Span<T>) -> MathElement
where
    HtmlNode: From<T>,
    T: Clone,
{
    // VList structures in KaTeX are complex table-based layouts
    // For now, create a VBox with positioned children
    let mut children = Vec::new();

    for child in &span.children {
        let html_child: HtmlNode = child.clone().into();
        let element = convert_node(&html_child);

        // Extract vertical position from style.top if available
        let y_offset = html_child
            .node()
            .style
            .top
            .as_ref()
            .and_then(|s| parse_em(s))
            .map(|v| -v) // top is inverted (CSS top offset → baseline-relative)
            .unwrap_or(0.0);

        children.push(Positioned::new(element, 0.0, y_offset));
    }

    MathElement::VBox {
        children,
        width: span.width.unwrap_or(0.0),
        height: span.node.height,
        depth: span.node.depth,
    }
}

/// Infer font from CSS classes
fn infer_font_from_classes(classes: &[String]) -> Option<Font> {
    for class in classes {
        match class.as_str() {
            "mathnormal" => return Some(Font::MathItalic),
            "mathbf" => return Some(Font::MainBold),
            "mathit" | "textit" => return Some(Font::MainItalic),
            "mathrm" | "textrm" => return Some(Font::MainRegular),
            "mathbb" => return Some(Font::AmsRegular),
            "mathcal" => return Some(Font::CaligraphicRegular),
            "mathfrak" => return Some(Font::FrakturRegular),
            "mathscr" => return Some(Font::ScriptRegular),
            "mathsf" | "textsf" => return Some(Font::SansSerifRegular),
            "mathtt" | "texttt" => return Some(Font::TypewriterRegular),
            "boldsymbol" => return Some(Font::MathBoldItalic),
            "amsrm" => return Some(Font::AmsRegular),
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_font() {
        let classes = vec!["mathnormal".to_string()];
        assert_eq!(infer_font_from_classes(&classes), Some(Font::MathItalic));

        let classes = vec!["mathbf".to_string()];
        assert_eq!(infer_font_from_classes(&classes), Some(Font::MainBold));

        let classes = vec!["unknown".to_string()];
        assert_eq!(infer_font_from_classes(&classes), None);
    }
}
