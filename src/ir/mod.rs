//! Intermediate Representation (IR) for math layout.
//!
//! This module provides a backend-agnostic representation of rendered math.
//! The IR can be converted to HTML, MathML, or any custom rendering backend
//! (such as GPUI for Zed editor integration).
//!
//! # Architecture
//!
//! There are two ways to produce IR:
//!
//! ## 1. Native IR Builder (Recommended)
//!
//! Build IR directly from parse trees with accurate metrics:
//!
//! ```text
//! LaTeX input → Parser → ParseNode tree → IR Builder → MathLayout
//!                                              ↓
//!                              ┌───────────────┼───────────────┐
//!                              ↓               ↓               ↓
//!                         HTML output    MathML output   Custom backend
//! ```
//!
//! ## 2. HTML Conversion (Legacy)
//!
//! Convert from HTML output (less accurate, for compatibility):
//!
//! ```text
//! LaTeX input → Parser → ParseNode tree → HTML Builder → HtmlNode → IR
//! ```
//!
//! # Semantic vs Layout Mode
//!
//! The IR can operate in two modes:
//!
//! - **Semantic mode** (default): Produces `MathElement` variants like `Fraction`,
//!   `Scripts`, `Radical` that preserve mathematical structure. Each semantic
//!   variant includes a pre-computed `layout` field for generic rendering.
//!
//! - **Layout mode**: Produces pure layout primitives (`HBox`, `VBox`, `Text`, etc.)
//!   without semantic information.
//!
//! # Example
//!
//! ```ignore
//! use aliter::{parse_tree, ir, parser::ParserConfig, Options};
//!
//! // Native IR builder (recommended)
//! let tree = parse_tree(r"\frac{x^2}{y}", ParserConfig::default())?;
//! let options = Options::from_parser_conf(&ParserConfig::default());
//! let layout = ir::builder::build_ir(&tree, &options);
//!
//! // Access semantic structure
//! if let ir::MathElement::Fraction { numerator, denominator, bar, .. } = &layout.root {
//!     println!("Numerator y-offset: {}", numerator.y);
//!     println!("Bar thickness: {:?}", bar.as_ref().map(|b| b.thickness));
//! }
//!
//! // Or walk all elements
//! for item in layout.walk() {
//!     match item.element {
//!         ir::MathElement::Text { text, style } => {
//!             println!("Text '{}' at ({}, {})", text, item.abs_x, item.abs_y);
//!         }
//!         _ => {}
//!     }
//! }
//!
//! // Convert to HTML
//! let html = ir::to_html::render(&layout);
//! ```

// Core types
mod types;
pub use types::*;

// Native IR builder
pub mod builder;
pub use builder::{build_ir, build_ir_with_config, IrBuilderConfig};

// HTML conversion (for compatibility)
#[cfg(feature = "html")]
pub mod from_html;

#[cfg(feature = "html")]
pub mod to_html;

#[cfg(test)]
mod tests {
    use super::*;

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
}
