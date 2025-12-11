//! Core types for the IR (Intermediate Representation) module.
//!
//! All dimensions are in em units unless otherwise specified.

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
    /// Actual width from font metrics (if known), pre-scaled by size
    pub width: Option<f64>,
    /// Height above baseline from font metrics (if known), pre-scaled by size
    pub height: Option<f64>,
    /// Depth below baseline from font metrics (if known), pre-scaled by size
    pub depth: Option<f64>,
}

/// Line style for rules/strokes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineStyle {
    #[default]
    Solid,
    Dashed,
}

/// Information about a fraction bar.
#[derive(Debug, Clone, PartialEq)]
pub struct FractionBar {
    /// Thickness of the bar in em
    pub thickness: f64,
    /// Optional color override
    pub color: Option<Color>,
    /// Line style (solid or dashed)
    pub style: LineStyle,
}

/// The core math layout element types.
///
/// All dimensions are in em units.
///
/// This enum has two categories of variants:
/// 1. **Layout variants** (Text, HBox, VBox, etc.) - Pure layout primitives
/// 2. **Semantic variants** (Fraction, Scripts, etc.) - Preserve semantic structure
///
/// Semantic variants contain both semantic information AND a pre-computed `layout` field.
/// Renderers can choose to:
/// - Use semantic fields for native math rendering
/// - Use the `layout` field for generic box-based rendering
#[derive(Debug, Clone, PartialEq)]
pub enum MathElement {
    // =========================================================================
    // Layout Primitives
    // =========================================================================

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

    // =========================================================================
    // Semantic Variants
    // These preserve mathematical structure while also providing layout.
    // =========================================================================

    /// A fraction with numerator, denominator, and optional bar.
    ///
    /// Includes `\frac`, `\dfrac`, `\tfrac`, `\binom`, etc.
    Fraction {
        /// The numerator, positioned above the bar
        numerator: Box<Positioned<MathElement>>,
        /// The denominator, positioned below the bar
        denominator: Box<Positioned<MathElement>>,
        /// The fraction bar (None for binomials/atop)
        bar: Option<FractionBar>,
        /// Pre-computed layout for generic rendering
        layout: Box<MathElement>,
    },

    /// Superscript and/or subscript attached to a base.
    ///
    /// The `Positioned` wrappers contain explicit y-offsets computed from
    /// font metrics (sup1, sub1, etc.), not CSS class detection.
    Scripts {
        /// The base element (may be None for pure scripts like `{}^2`)
        base: Option<Box<Positioned<MathElement>>>,
        /// Superscript with computed y-offset (positive = up)
        superscript: Option<Box<Positioned<MathElement>>>,
        /// Subscript with computed y-offset (negative = down)
        subscript: Option<Box<Positioned<MathElement>>>,
        /// Pre-computed layout for generic rendering
        layout: Box<MathElement>,
    },

    /// A square root or nth root.
    Radical {
        /// The expression under the radical
        radicand: Box<MathElement>,
        /// Optional index (for nth roots like \sqrt[3]{x})
        index: Option<Box<MathElement>>,
        /// Pre-computed layout including surd symbol and vinculum
        layout: Box<MathElement>,
    },

    /// An accent over or under a base expression.
    ///
    /// Includes `\hat`, `\tilde`, `\vec`, `\bar`, `\overline`, `\underline`, etc.
    Accent {
        /// The base expression being accented
        base: Box<MathElement>,
        /// The accent symbol or line
        accent: Box<MathElement>,
        /// True if accent is above base, false if below
        is_over: bool,
        /// Pre-computed layout
        layout: Box<MathElement>,
    },

    /// A delimited expression with \left...\right or similar.
    Delimited {
        /// Left delimiter (may be None for \left.)
        left: Option<Box<MathElement>>,
        /// Right delimiter (may be None for \right.)
        right: Option<Box<MathElement>>,
        /// The body between delimiters
        body: Box<MathElement>,
        /// Pre-computed layout
        layout: Box<MathElement>,
    },

    /// A large operator (sum, product, integral, etc.) with optional limits.
    LargeOp {
        /// The operator symbol
        nucleus: Box<MathElement>,
        /// Superscript/upper limit
        superscript: Option<Box<Positioned<MathElement>>>,
        /// Subscript/lower limit
        subscript: Option<Box<Positioned<MathElement>>>,
        /// True if limits are above/below, false if to the side
        limits: bool,
        /// Pre-computed layout
        layout: Box<MathElement>,
    },

    /// A matrix or array structure.
    Array {
        /// Rows of cells, each cell is a MathElement
        cells: Vec<Vec<MathElement>>,
        /// Row separator lines
        row_lines: Vec<bool>,
        /// Column separator lines
        col_lines: Vec<bool>,
        /// Pre-computed layout
        layout: Box<MathElement>,
    },
}

impl MathElement {
    /// Get the bounding box dimensions of this element.
    pub fn dimensions(&self) -> (f64, f64, f64) {
        match self {
            MathElement::Text { style, text } => {
                // Use stored metrics if available, otherwise approximate.
                // All metrics are pre-scaled by size in the builder.
                let size = style.size.max(1.0);
                let width = style.width.unwrap_or_else(|| {
                    text.chars().count() as f64 * 0.5 * size
                });
                // Default height/depth based on typical KaTeX_Main metrics:
                // ascent ≈ 0.656, descent ≈ 0.219
                let height = style.height.unwrap_or(0.656 * size);
                let depth = style.depth.unwrap_or(0.219 * size);
                (width, height, depth)
            }
            MathElement::HBox { width, height, depth, .. } => (*width, *height, *depth),
            MathElement::VBox { width, height, depth, .. } => (*width, *height, *depth),
            MathElement::Rule { width, height, .. } => {
                // Rule dimensions are just its intrinsic size (width x height).
                // The shift field is metadata for renderers; positioning is handled
                // by the Positioned wrapper, not by inflating dimensions.
                (*width, *height, 0.0)
            }
            MathElement::Path { width, height, shift, .. } => {
                // For Path, shift encodes where the baseline sits within the element.
                // Positive shift = baseline is above center (element extends below)
                // Negative shift = baseline is below center (element extends above)
                // This is used for stretchy delimiters that need proper height/depth.
                (*width, *height + shift.max(0.0), (-*shift).max(0.0))
            }
            MathElement::Kern { width } => (*width, 0.0, 0.0),
            MathElement::Phantom { inner } => inner.dimensions(),
            MathElement::Color { inner, .. } => inner.dimensions(),
            MathElement::Link { inner, .. } => inner.dimensions(),
            MathElement::Image { width, height, .. } => (*width, *height, 0.0),
            MathElement::Breakable { width, height, depth, .. } => (*width, *height, *depth),

            // Semantic variants delegate to their layout
            MathElement::Fraction { layout, .. } => layout.dimensions(),
            MathElement::Scripts { layout, .. } => layout.dimensions(),
            MathElement::Radical { layout, .. } => layout.dimensions(),
            MathElement::Accent { layout, .. } => layout.dimensions(),
            MathElement::Delimited { layout, .. } => layout.dimensions(),
            MathElement::LargeOp { layout, .. } => layout.dimensions(),
            MathElement::Array { layout, .. } => layout.dimensions(),
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

    /// Returns true if this is a semantic variant.
    pub fn is_semantic(&self) -> bool {
        matches!(
            self,
            MathElement::Fraction { .. }
                | MathElement::Scripts { .. }
                | MathElement::Radical { .. }
                | MathElement::Accent { .. }
                | MathElement::Delimited { .. }
                | MathElement::LargeOp { .. }
                | MathElement::Array { .. }
        )
    }

    /// Get the inner layout element for semantic variants.
    /// Returns self for non-semantic variants.
    pub fn as_layout(&self) -> &MathElement {
        match self {
            MathElement::Fraction { layout, .. } => layout.as_ref(),
            MathElement::Scripts { layout, .. } => layout.as_ref(),
            MathElement::Radical { layout, .. } => layout.as_ref(),
            MathElement::Accent { layout, .. } => layout.as_ref(),
            MathElement::Delimited { layout, .. } => layout.as_ref(),
            MathElement::LargeOp { layout, .. } => layout.as_ref(),
            MathElement::Array { layout, .. } => layout.as_ref(),
            _ => self,
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
            // For semantic variants, walk the layout
            MathElement::Fraction { layout, .. }
            | MathElement::Scripts { layout, .. }
            | MathElement::Radical { layout, .. }
            | MathElement::Accent { layout, .. }
            | MathElement::Delimited { layout, .. }
            | MathElement::LargeOp { layout, .. }
            | MathElement::Array { layout, .. } => {
                self.stack.push((abs_x, abs_y, layout.as_ref()));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positioned() {
        let pos = Positioned::new(MathElement::Kern { width: 1.0 }, 2.0, 3.0);
        assert_eq!(pos.x, 2.0);
        assert_eq!(pos.y, 3.0);
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
        // Rule dimensions are intrinsic size only - shift is metadata for renderers,
        // not included in the bounding box (positioning handles placement)
        let rule = MathElement::Rule {
            width: 10.0,
            height: 0.04,
            shift: 0.25,
            style: LineStyle::Solid,
            color: None,
        };
        let (w, h, d) = rule.dimensions();
        assert_eq!(w, 10.0);
        assert_eq!(h, 0.04); // Just the intrinsic height
        assert_eq!(d, 0.0);  // Rules sit above baseline

        // Path uses shift to encode baseline position within the element
        // (different semantics from Rule - used for stretchy delimiters)
        let path = MathElement::Path {
            path_data: std::borrow::Cow::Borrowed("test"),
            width: 1.0,
            height: 2.0,
            shift: -0.5, // Baseline is 0.5 below center, so depth = 0.5
        };
        let (w, h, d) = path.dimensions();
        assert_eq!(w, 1.0);
        assert_eq!(h, 2.0); // height + (-0.5).max(0) = 2.0
        assert_eq!(d, 0.5); // (-(-0.5)).max(0) = 0.5

        // Positive shift means baseline above center
        let path2 = MathElement::Path {
            path_data: std::borrow::Cow::Borrowed("test"),
            width: 1.0,
            height: 2.0,
            shift: 0.5, // Baseline is 0.5 above center
        };
        let (w, h, d) = path2.dimensions();
        assert_eq!(w, 1.0);
        assert_eq!(h, 2.5); // height + 0.5
        assert_eq!(d, 0.0); // No depth when shift is positive
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

    #[test]
    fn test_semantic_variant_dimensions() {
        // Create a simple fraction
        let numer = MathElement::Text {
            text: "1".to_string(),
            style: TextStyle { size: 1.0, width: Some(0.5), ..Default::default() },
        };
        let denom = MathElement::Text {
            text: "2".to_string(),
            style: TextStyle { size: 1.0, width: Some(0.5), ..Default::default() },
        };
        let layout = MathElement::VBox {
            children: vec![],
            width: 0.5,
            height: 1.0,
            depth: 0.5,
        };

        let frac = MathElement::Fraction {
            numerator: Box::new(Positioned::new(numer, 0.0, 0.5)),
            denominator: Box::new(Positioned::new(denom, 0.0, -0.5)),
            bar: Some(FractionBar {
                thickness: 0.04,
                color: None,
                style: LineStyle::Solid,
            }),
            layout: Box::new(layout),
        };

        assert!(frac.is_semantic());
        let (w, h, d) = frac.dimensions();
        assert_eq!(w, 0.5);
        assert_eq!(h, 1.0);
        assert_eq!(d, 0.5);
    }
}
