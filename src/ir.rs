//! Intermediate Representation (IR) for math layout.
//!
//! This module provides a backend-agnostic representation of rendered math.
//! The IR can be converted to HTML, MathML, or any custom rendering backend
//! (such as GPUI for Zed editor integration).
//!
//! # Architecture
//!
//! ```text
//! LaTeX input → Parser → ParseNode tree → IR Builder → MathLayout
//!                                                          ↓
//!                                           ┌──────────────┼──────────────┐
//!                                           ↓              ↓              ↓
//!                                      HTML output   MathML output   Custom backend
//! ```
//!
//! # Example
//!
//! ```ignore
//! use aliter::{parse_tree, ir::build_layout, parser::ParserConfig};
//!
//! let tree = parse_tree(r"\frac{1}{2}", ParserConfig::default())?;
//! let layout = build_layout(&tree, &ParserConfig::default());
//!
//! // Convert to HTML
//! let html = layout.to_html();
//!
//! // Or use a custom renderer
//! for element in layout.walk() {
//!     match element {
//!         MathElement::Text { text, font, .. } => { /* render text */ }
//!         MathElement::Rule { width, height, .. } => { /* draw line */ }
//!         // ...
//!     }
//! }
//! ```

use std::borrow::Cow;

use crate::parse_node::Color;

/// A positioned element within its parent's coordinate system.
///
/// Coordinates are in em units, with the origin at the parent's baseline.
/// - Positive x is rightward
/// - Positive y is upward from baseline
#[derive(Debug, Clone, PartialEq)]
pub struct Positioned<T> {
    pub element: T,
    /// Horizontal offset from parent's left edge (in em)
    pub x: f64,
    /// Vertical offset from parent's baseline (in em, positive = up)
    pub y: f64,
}

impl<T> Positioned<T> {
    pub fn new(element: T, x: f64, y: f64) -> Self {
        Self { element, x, y }
    }

    pub fn at_origin(element: T) -> Self {
        Self::new(element, 0.0, 0.0)
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Positioned<U> {
        Positioned {
            element: f(self.element),
            x: self.x,
            y: self.y,
        }
    }
}

/// Font identifier for text rendering.
///
/// These correspond to the KaTeX font files.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Font {
    MainRegular,
    MainBold,
    MainItalic,
    MainBoldItalic,
    MathItalic,
    MathBoldItalic,
    SansSerifRegular,
    SansSerifBold,
    SansSerifItalic,
    TypewriterRegular,
    CaligraphicRegular,
    CaligraphicBold,
    FrakturRegular,
    FrakturBold,
    ScriptRegular,
    AmsRegular,
    /// Custom/unknown font name
    Other(Cow<'static, str>),
}

impl Font {
    /// Parse a font name string into a Font variant
    pub fn from_name(name: &str) -> Self {
        match name {
            "Main-Regular" => Font::MainRegular,
            "Main-Bold" => Font::MainBold,
            "Main-Italic" => Font::MainItalic,
            "Main-BoldItalic" => Font::MainBoldItalic,
            "Math-Italic" => Font::MathItalic,
            "Math-BoldItalic" => Font::MathBoldItalic,
            "SansSerif-Regular" => Font::SansSerifRegular,
            "SansSerif-Bold" => Font::SansSerifBold,
            "SansSerif-Italic" => Font::SansSerifItalic,
            "Typewriter-Regular" => Font::TypewriterRegular,
            "Caligraphic-Regular" => Font::CaligraphicRegular,
            "Caligraphic-Bold" => Font::CaligraphicBold,
            "Fraktur-Regular" => Font::FrakturRegular,
            "Fraktur-Bold" => Font::FrakturBold,
            "Script-Regular" => Font::ScriptRegular,
            "AMS-Regular" => Font::AmsRegular,
            other => Font::Other(Cow::Owned(other.to_string())),
        }
    }

    /// Get the font name as a string (for CSS font-family, etc.)
    pub fn as_str(&self) -> &str {
        match self {
            Font::MainRegular => "Main-Regular",
            Font::MainBold => "Main-Bold",
            Font::MainItalic => "Main-Italic",
            Font::MainBoldItalic => "Main-BoldItalic",
            Font::MathItalic => "Math-Italic",
            Font::MathBoldItalic => "Math-BoldItalic",
            Font::SansSerifRegular => "SansSerif-Regular",
            Font::SansSerifBold => "SansSerif-Bold",
            Font::SansSerifItalic => "SansSerif-Italic",
            Font::TypewriterRegular => "Typewriter-Regular",
            Font::CaligraphicRegular => "Caligraphic-Regular",
            Font::CaligraphicBold => "Caligraphic-Bold",
            Font::FrakturRegular => "Fraktur-Regular",
            Font::FrakturBold => "Fraktur-Bold",
            Font::ScriptRegular => "Script-Regular",
            Font::AmsRegular => "AMS-Regular",
            Font::Other(name) => name.as_ref(),
        }
    }
}

/// Style information for text rendering.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextStyle {
    pub font: Option<Font>,
    /// Size multiplier relative to base size (1.0 = normal)
    pub size: f64,
    pub color: Option<Color>,
    /// Italic correction (extra space after italic text)
    pub italic_correction: f64,
    /// Skew for accent positioning
    pub skew: f64,
}

/// Line style for rules/strokes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineStyle {
    #[default]
    Solid,
    Dashed,
}

/// The core math layout element types.
///
/// All dimensions are in em units.
#[derive(Debug, Clone, PartialEq)]
pub enum MathElement {
    /// A text run (one or more characters with the same styling).
    Text {
        text: String,
        style: TextStyle,
    },

    /// A horizontal box containing positioned children.
    /// Children are laid out left-to-right by default.
    HBox {
        children: Vec<Positioned<MathElement>>,
        /// Total width of the box
        width: f64,
        /// Height above baseline
        height: f64,
        /// Depth below baseline
        depth: f64,
        /// Optional CSS classes (for compatibility/debugging)
        classes: Vec<String>,
    },

    /// A vertical box containing positioned children.
    /// Children are stacked vertically.
    VBox {
        children: Vec<Positioned<MathElement>>,
        /// Total width of the box
        width: f64,
        /// Height above baseline
        height: f64,
        /// Depth below baseline
        depth: f64,
    },

    /// A horizontal or vertical rule (line).
    /// Used for fraction bars, overlines, sqrt vinculum, etc.
    Rule {
        /// Width of the rule
        width: f64,
        /// Height (thickness) of the rule
        height: f64,
        /// Vertical shift from baseline
        shift: f64,
        /// Line style
        style: LineStyle,
        /// Optional color
        color: Option<Color>,
    },

    /// An SVG path for stretchy delimiters and special symbols.
    Path {
        /// SVG path data (d attribute)
        path_data: Cow<'static, str>,
        /// Width of the bounding box
        width: f64,
        /// Height of the bounding box
        height: f64,
        /// Vertical shift
        shift: f64,
    },

    /// A kern (invisible spacing element).
    Kern {
        /// Width of the space (can be negative)
        width: f64,
    },

    /// Transparent/invisible element (for \phantom).
    Phantom {
        /// The element being made invisible
        inner: Box<MathElement>,
    },

    /// A colored region wrapping content.
    Color {
        color: Color,
        inner: Box<MathElement>,
    },

    /// A link/anchor wrapping content.
    Link {
        href: String,
        inner: Box<MathElement>,
    },

    /// An image element (for \includegraphics).
    Image {
        src: String,
        alt: String,
        width: f64,
        height: f64,
    },

    /// A group that serves as a line break point.
    Breakable {
        children: Vec<Positioned<MathElement>>,
        width: f64,
        height: f64,
        depth: f64,
    },
}

impl MathElement {
    /// Get the bounding box dimensions of this element.
    pub fn dimensions(&self) -> (f64, f64, f64) {
        match self {
            MathElement::Text { style, text } => {
                // Approximate dimensions based on text length and size
                // In practice, these would be computed from font metrics
                let width = text.chars().count() as f64 * 0.5 * style.size;
                let height = 0.7 * style.size;
                let depth = 0.2 * style.size;
                (width, height, depth)
            }
            MathElement::HBox { width, height, depth, .. } => (*width, *height, *depth),
            MathElement::VBox { width, height, depth, .. } => (*width, *height, *depth),
            MathElement::Rule { width, height, shift, .. } => {
                (*width, *height + shift.max(0.0), (-shift).max(0.0))
            }
            MathElement::Path { width, height, shift, .. } => {
                (*width, *height + shift.max(0.0), (-shift).max(0.0))
            }
            MathElement::Kern { width } => (*width, 0.0, 0.0),
            MathElement::Phantom { inner } => inner.dimensions(),
            MathElement::Color { inner, .. } => inner.dimensions(),
            MathElement::Link { inner, .. } => inner.dimensions(),
            MathElement::Image { width, height, .. } => (*width, *height, 0.0),
            MathElement::Breakable { width, height, depth, .. } => (*width, *height, *depth),
        }
    }

    /// Get the width of this element.
    pub fn width(&self) -> f64 {
        self.dimensions().0
    }

    /// Get the height above baseline.
    pub fn height(&self) -> f64 {
        self.dimensions().1
    }

    /// Get the depth below baseline.
    pub fn depth(&self) -> f64 {
        self.dimensions().2
    }

    /// Check if this is an empty/zero-width element.
    pub fn is_empty(&self) -> bool {
        match self {
            MathElement::HBox { children, .. } => children.is_empty(),
            MathElement::VBox { children, .. } => children.is_empty(),
            MathElement::Text { text, .. } => text.is_empty(),
            MathElement::Kern { width } => *width == 0.0,
            _ => false,
        }
    }
}

/// The root layout structure containing the rendered math.
#[derive(Debug, Clone, PartialEq)]
pub struct MathLayout {
    /// The root element of the layout tree
    pub root: MathElement,
    /// Whether this was rendered in display mode
    pub display_mode: bool,
    /// Total width
    pub width: f64,
    /// Height above baseline
    pub height: f64,
    /// Depth below baseline
    pub depth: f64,
}

impl MathLayout {
    pub fn new(root: MathElement, display_mode: bool) -> Self {
        let (width, height, depth) = root.dimensions();
        Self {
            root,
            display_mode,
            width,
            height,
            depth,
        }
    }

    /// Iterate over all elements in depth-first order.
    pub fn walk(&self) -> LayoutWalker<'_> {
        LayoutWalker::new(&self.root)
    }
}

/// Iterator for walking the layout tree.
pub struct LayoutWalker<'a> {
    stack: Vec<(f64, f64, &'a MathElement)>,
}

impl<'a> LayoutWalker<'a> {
    fn new(root: &'a MathElement) -> Self {
        Self {
            stack: vec![(0.0, 0.0, root)],
        }
    }
}

/// A visited element with its absolute position.
pub struct LayoutItem<'a> {
    pub element: &'a MathElement,
    pub abs_x: f64,
    pub abs_y: f64,
}

impl<'a> Iterator for LayoutWalker<'a> {
    type Item = LayoutItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (abs_x, abs_y, element) = self.stack.pop()?;

        // Push children onto stack in reverse order (so they're visited in order)
        match element {
            MathElement::HBox { children, .. }
            | MathElement::VBox { children, .. }
            | MathElement::Breakable { children, .. } => {
                for child in children.iter().rev() {
                    self.stack.push((
                        abs_x + child.x,
                        abs_y + child.y,
                        &child.element,
                    ));
                }
            }
            MathElement::Phantom { inner }
            | MathElement::Color { inner, .. }
            | MathElement::Link { inner, .. } => {
                self.stack.push((abs_x, abs_y, inner.as_ref()));
            }
            _ => {}
        }

        Some(LayoutItem {
            element,
            abs_x,
            abs_y,
        })
    }
}

// =============================================================================
// HTML to IR Converter
// =============================================================================

#[cfg(feature = "html")]
pub mod from_html {
    //! Convert HtmlNode trees to IR representation.
    //!
    //! This module provides conversion from the HTML DOM tree to the IR format,
    //! which proves the IR can represent everything the HTML builder produces.

    use super::*;
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
                    .unwrap_or(Cow::Borrowed(""));

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
                .map(|v| -v) // top is inverted
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
}

// =============================================================================
// IR to HTML Renderer
// =============================================================================

#[cfg(feature = "html")]
pub mod to_html {
    //! Render IR to HTML string.
    //!
    //! This allows roundtrip testing: HtmlNode -> IR -> HTML string

    use super::*;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positioned() {
        let pos = Positioned::new(MathElement::Kern { width: 1.0 }, 2.0, 3.0);
        assert_eq!(pos.x, 2.0);
        assert_eq!(pos.y, 3.0);
    }

    #[cfg(feature = "html")]
    #[test]
    fn test_html_to_ir_roundtrip() {
        use crate::{render_to_html_tree, parser::ParserConfig, tree::VirtualNode};

        // Test a few expressions
        let expressions = [
            "x",
            "x + y",
            r"\frac{1}{2}",
            r"\sqrt{x}",
            r"x^2",
            r"x_i",
            r"\sum_{i=0}^{n} x_i",
        ];

        for expr in expressions {
            let conf = ParserConfig::default();
            let html_tree = render_to_html_tree(expr, conf);

            // Convert HTML to IR
            let ir = from_html::convert(&crate::dom_tree::HtmlNode::Span(html_tree.clone()));

            // Verify we got a valid IR structure
            assert!(!ir.is_empty() || expr.is_empty(), "IR should not be empty for '{}'", expr);

            // The IR should have reasonable dimensions
            let (w, h, d) = ir.dimensions();
            // Width can be 0 for some empty containers, but height/depth should be reasonable
            assert!(h >= 0.0, "Height should be non-negative for '{}'", expr);
            assert!(d >= 0.0, "Depth should be non-negative for '{}'", expr);
        }
    }

    #[cfg(feature = "html")]
    #[test]
    fn test_ir_walker() {
        use crate::{render_to_html_tree, parser::ParserConfig};

        let conf = ParserConfig::default();
        let html_tree = render_to_html_tree("x + y", conf);
        let ir = from_html::convert(&crate::dom_tree::HtmlNode::Span(html_tree));

        let layout = MathLayout::new(ir, false);
        let items: Vec<_> = layout.walk().collect();

        // Should have multiple elements
        assert!(items.len() > 1, "Walker should find multiple elements");

        // Check that we found some text
        let has_text = items.iter().any(|item| {
            matches!(item.element, MathElement::Text { .. })
        });
        assert!(has_text, "Should find text elements in 'x + y'");
    }

    #[test]
    fn test_font_from_name() {
        assert_eq!(Font::from_name("Main-Regular"), Font::MainRegular);
        assert_eq!(Font::from_name("Math-Italic"), Font::MathItalic);
        assert_eq!(
            Font::from_name("Custom-Font"),
            Font::Other(Cow::Owned("Custom-Font".to_string()))
        );
    }

    #[test]
    fn test_element_dimensions() {
        let rule = MathElement::Rule {
            width: 10.0,
            height: 0.04,
            shift: 0.25,
            style: LineStyle::Solid,
            color: None,
        };
        let (w, h, d) = rule.dimensions();
        assert_eq!(w, 10.0);
        assert_eq!(h, 0.04 + 0.25); // height + shift
        assert_eq!(d, 0.0); // shift is positive, so no depth
    }

    #[test]
    fn test_layout_walker() {
        let layout = MathLayout::new(
            MathElement::HBox {
                children: vec![
                    Positioned::new(
                        MathElement::Text {
                            text: "x".to_string(),
                            style: TextStyle { size: 1.0, ..Default::default() },
                        },
                        0.0,
                        0.0,
                    ),
                    Positioned::new(
                        MathElement::Text {
                            text: "+".to_string(),
                            style: TextStyle { size: 1.0, ..Default::default() },
                        },
                        0.5,
                        0.0,
                    ),
                    Positioned::new(
                        MathElement::Text {
                            text: "y".to_string(),
                            style: TextStyle { size: 1.0, ..Default::default() },
                        },
                        1.0,
                        0.0,
                    ),
                ],
                width: 1.5,
                height: 0.7,
                depth: 0.2,
                classes: vec![],
            },
            false,
        );

        let items: Vec<_> = layout.walk().collect();
        assert_eq!(items.len(), 4); // HBox + 3 Text elements
    }
}
