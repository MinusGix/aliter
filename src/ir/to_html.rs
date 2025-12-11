//! Render IR to HTML string.
//!
//! This allows roundtrip testing: HtmlNode -> IR -> HTML string

use super::types::*;
use crate::unit::make_em;

/// Render a MathLayout to an HTML string.
pub fn render(layout: &MathLayout) -> String {
    let mut out = String::new();

    if layout.display_mode {
        out.push_str("<span class=\"katex-display\">");
    }
    out.push_str("<span class=\"katex\">");
    out.push_str("<span class=\"katex-html\" aria-hidden=\"true\">");

    render_element(&layout.root, &mut out);

    out.push_str("</span>");
    out.push_str("</span>");
    if layout.display_mode {
        out.push_str("</span>");
    }

    out
}

fn render_element(element: &MathElement, out: &mut String) {
    match element {
        MathElement::Text { text, style } => {
            let needs_span = style.color.is_some()
                || style.italic_correction > 0.0
                || style.font.is_some();

            if needs_span {
                out.push_str("<span");

                // Build class list
                let mut classes = Vec::new();
                if let Some(font) = &style.font {
                    classes.push(font_to_class(font));
                }
                if !classes.is_empty() {
                    out.push_str(" class=\"");
                    out.push_str(&classes.join(" "));
                    out.push('"');
                }

                // Build style
                let mut styles = String::new();
                if let Some(color) = &style.color {
                    styles.push_str(&format!("color:{};", color.to_string()));
                }
                if style.italic_correction > 0.0 {
                    styles.push_str(&format!("margin-right:{};", make_em(style.italic_correction)));
                }
                if !styles.is_empty() {
                    out.push_str(" style=\"");
                    out.push_str(&styles);
                    out.push('"');
                }

                out.push('>');
                out.push_str(&html_escape(text));
                out.push_str("</span>");
            } else {
                out.push_str(&html_escape(text));
            }
        }

        MathElement::HBox { children, classes, .. } => {
            out.push_str("<span");
            if !classes.is_empty() {
                out.push_str(" class=\"");
                out.push_str(&classes.join(" "));
                out.push('"');
            }
            out.push('>');
            for child in children {
                render_element(&child.element, out);
            }
            out.push_str("</span>");
        }

        MathElement::VBox { children, .. } => {
            out.push_str("<span class=\"vlist\">");
            for child in children {
                out.push_str("<span");
                if child.y != 0.0 {
                    out.push_str(&format!(" style=\"top:{}\"", make_em(-child.y)));
                }
                out.push('>');
                render_element(&child.element, out);
                out.push_str("</span>");
            }
            out.push_str("</span>");
        }

        MathElement::Rule { width, height, style: line_style, color, .. } => {
            out.push_str("<span class=\"");
            out.push_str(match line_style {
                LineStyle::Solid => "rule",
                LineStyle::Dashed => "hdashline",
            });
            out.push_str("\" style=\"");
            out.push_str(&format!("width:{};", make_em(*width)));
            out.push_str(&format!("border-bottom-width:{};", make_em(*height)));
            if let Some(color) = color {
                out.push_str(&format!("border-color:{};", color.to_string()));
            }
            out.push_str("\"></span>");
        }

        MathElement::Path { path_data, width, height, .. } => {
            out.push_str(&format!(
                "<svg width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\"><path d=\"{}\"/></svg>",
                make_em(*width),
                make_em(*height),
                width * 1000.0,
                height * 1000.0,
                path_data
            ));
        }

        MathElement::Kern { width } => {
            if *width != 0.0 {
                out.push_str(&format!(
                    "<span class=\"mspace\" style=\"margin-right:{}\"></span>",
                    make_em(*width)
                ));
            }
        }

        MathElement::Phantom { inner } => {
            out.push_str("<span class=\"mord\" style=\"color:transparent\">");
            render_element(inner, out);
            out.push_str("</span>");
        }

        MathElement::Color { color, inner } => {
            out.push_str(&format!("<span style=\"color:{}\">", color.to_string()));
            render_element(inner, out);
            out.push_str("</span>");
        }

        MathElement::Link { href, inner } => {
            out.push_str(&format!("<a href=\"{}\">", html_escape(href)));
            render_element(inner, out);
            out.push_str("</a>");
        }

        MathElement::Image { src, alt, width, height } => {
            out.push_str(&format!(
                "<img src=\"{}\" alt=\"{}\" width=\"{}\" height=\"{}\"/>",
                html_escape(src),
                html_escape(alt),
                make_em(*width),
                make_em(*height)
            ));
        }

        MathElement::Breakable { children, .. } => {
            // Breakable is similar to HBox for HTML output
            out.push_str("<span class=\"base\">");
            for child in children {
                render_element(&child.element, out);
            }
            out.push_str("</span>");
        }

        // Semantic variants render their layout
        MathElement::Fraction { layout, .. }
        | MathElement::Scripts { layout, .. }
        | MathElement::Radical { layout, .. }
        | MathElement::Accent { layout, .. }
        | MathElement::Delimited { layout, .. }
        | MathElement::LargeOp { layout, .. }
        | MathElement::Array { layout, .. } => {
            render_element(layout, out);
        }
    }
}

fn font_to_class(font: &Font) -> &'static str {
    match font {
        Font::MainRegular => "textrm",
        Font::MainBold => "mathbf",
        Font::MainItalic => "textit",
        Font::MainBoldItalic => "mathbf",
        Font::MathItalic => "mathnormal",
        Font::MathBoldItalic => "boldsymbol",
        Font::SansSerifRegular => "mathsf",
        Font::SansSerifBold => "mathsf",
        Font::SansSerifItalic => "mathsf",
        Font::TypewriterRegular => "mathtt",
        Font::CaligraphicRegular | Font::CaligraphicBold => "mathcal",
        Font::FrakturRegular | Font::FrakturBold => "mathfrak",
        Font::ScriptRegular => "mathscr",
        Font::AmsRegular => "amsrm",
        Font::Other(_) => "",
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
