//! Native IR builder - builds MathLayout directly from ParseNode trees.
//!
//! This module bypasses the HTML intermediate representation to produce
//! IR with:
//! - Accurate widths from font metrics
//! - Explicit y-offsets for all positioned elements
//! - Optional semantic structure preservation
//!
//! # Example
//!
//! ```ignore
//! use aliter::{parse_tree, ir::builder::build_ir, parser::ParserConfig};
//!
//! let tree = parse_tree(r"\frac{1}{2}", ParserConfig::default())?;
//! let options = Options::from_parser_conf(&ParserConfig::default());
//! let layout = build_ir(&tree, &options);
//!
//! // layout.root contains MathElement::Fraction with explicit positioning
//! ```

use crate::expander::Mode;
use crate::font_metrics::{get_character_metrics, CharacterMetrics, FontMetrics};
use crate::parse_node::*;
use crate::style::DISPLAY_STYLE;
use crate::symbols;
use crate::unit::calculate_size;
use crate::Options;

use super::types::*;

/// Configuration for the IR builder.
#[derive(Debug, Clone)]
pub struct IrBuilderConfig {
    /// Whether to emit semantic variants (Fraction, Scripts, etc.)
    /// or pure layout variants (HBox, VBox, etc.).
    ///
    /// Default: true
    pub semantic_mode: bool,

    /// Whether to include CSS classes in HBox elements.
    /// Useful for debugging or HTML compatibility.
    ///
    /// Default: false
    pub include_classes: bool,
}

impl Default for IrBuilderConfig {
    fn default() -> Self {
        Self {
            semantic_mode: true,
            include_classes: false,
        }
    }
}

impl IrBuilderConfig {
    /// Create a config that produces pure layout (no semantic variants).
    pub fn layout_only() -> Self {
        Self {
            semantic_mode: false,
            include_classes: false,
        }
    }

    /// Create a config that includes CSS classes for debugging.
    pub fn with_classes() -> Self {
        Self {
            semantic_mode: true,
            include_classes: true,
        }
    }
}

/// Context for IR layout computation.
pub struct LayoutContext<'a> {
    pub options: &'a Options,
    pub config: &'a IrBuilderConfig,
}

impl<'a> LayoutContext<'a> {
    pub fn new(options: &'a Options, config: &'a IrBuilderConfig) -> Self {
        Self { options, config }
    }

    /// Get global font metrics for current style.
    pub fn metrics(&self) -> &FontMetrics {
        self.options.font_metrics()
    }

    /// Get character metrics for a symbol in the given font.
    pub fn char_metrics(&self, ch: char, font: &str, mode: Mode) -> Option<CharacterMetrics> {
        get_character_metrics(ch, font, mode)
    }

    /// Get the current mode (Math or Text) from Options.
    pub fn mode(&self) -> Mode {
        // Default to Math mode; Options doesn't directly store mode
        Mode::Math
    }

    /// Check if we're in display style.
    pub fn is_display(&self) -> bool {
        self.options.style.size() == DISPLAY_STYLE.size()
    }

    /// Check if the current style is cramped.
    pub fn is_cramped(&self) -> bool {
        self.options.style.cramped()
    }

    /// Get the current size multiplier.
    pub fn size_multiplier(&self) -> f64 {
        self.options.size_multiplier()
    }

    /// Create a child context with a different style.
    pub fn with_style(&self, _style: crate::style::StyleId) -> LayoutContext<'a> {
        // For now we create a temporary Options. In the future we might
        // want to optimize this to actually apply the style change.
        LayoutContext {
            options: self.options,
            config: self.config,
        }
    }
}

// =============================================================================
// Public API
// =============================================================================

/// Build IR directly from a parse tree using default configuration.
pub fn build_ir(tree: &[ParseNode], options: &Options) -> MathLayout {
    build_ir_with_config(tree, options, &IrBuilderConfig::default())
}

/// Build IR directly from a parse tree with custom configuration.
pub fn build_ir_with_config(
    tree: &[ParseNode],
    options: &Options,
    config: &IrBuilderConfig,
) -> MathLayout {
    let ctx = LayoutContext::new(options, config);
    let root = build_expression(tree, &ctx);
    let is_display = options.style.size() == DISPLAY_STYLE.size();
    MathLayout::new(root, is_display)
}

// =============================================================================
// Expression Builder
// =============================================================================

/// Build an expression (sequence of nodes) into an HBox.
fn build_expression(nodes: &[ParseNode], ctx: &LayoutContext) -> MathElement {
    if nodes.is_empty() {
        return MathElement::HBox {
            children: vec![],
            width: 0.0,
            height: 0.0,
            depth: 0.0,
            classes: vec![],
        };
    }

    let mut children = Vec::new();
    let mut x_offset = 0.0;
    let mut max_height = 0.0f64;
    let mut max_depth = 0.0f64;

    for node in nodes {
        let element = build_node(node, ctx);
        let (width, height, depth) = element.dimensions();

        children.push(Positioned::new(element, x_offset, 0.0));
        x_offset += width;
        max_height = max_height.max(height);
        max_depth = max_depth.max(depth);

        // TODO: Add inter-element spacing based on atom types
        // (port spacing logic from html.rs)
    }

    MathElement::HBox {
        children,
        width: x_offset,
        height: max_height,
        depth: max_depth,
        classes: if ctx.config.include_classes {
            vec!["base".to_string()]
        } else {
            vec![]
        },
    }
}

// =============================================================================
// Node Builders
// =============================================================================

/// Build a single parse node into an IR element.
fn build_node(node: &ParseNode, ctx: &LayoutContext) -> MathElement {
    match node {
        ParseNode::MathOrd(ord) => build_math_ord(ord, ctx),
        ParseNode::TextOrd(ord) => build_text_ord(ord, ctx),
        ParseNode::Atom(atom) => build_atom(atom, ctx),
        ParseNode::Spacing(sp) => build_spacing(sp, ctx),
        ParseNode::Kern(kern) => build_kern(kern, ctx),
        ParseNode::OrdGroup(group) => build_expression(&group.body, ctx),
        ParseNode::SupSub(supsub) => build_supsub(supsub, ctx),
        ParseNode::GenFrac(frac) => build_fraction(frac, ctx),
        ParseNode::Sqrt(sqrt) => build_sqrt(sqrt, ctx),
        ParseNode::Accent(accent) => build_accent(accent, ctx),
        ParseNode::Op(op) => build_op(op, ctx),
        ParseNode::Color(color) => build_color(color, ctx),
        ParseNode::Font(font) => build_font(font, ctx),
        ParseNode::Styling(styling) => build_styling(styling, ctx),
        ParseNode::Sizing(sizing) => build_sizing(sizing, ctx),
        ParseNode::Overline(over) => build_overline(over, ctx),
        ParseNode::Underline(under) => build_underline(under, ctx),
        ParseNode::Phantom(phantom) => build_phantom(phantom, ctx),
        ParseNode::HPhantom(hphantom) => build_hphantom(hphantom, ctx),
        ParseNode::VPhantom(vphantom) => build_vphantom(vphantom, ctx),
        ParseNode::Rule(rule) => build_rule(rule, ctx),
        ParseNode::LeftRight(lr) => build_left_right(lr, ctx),
        ParseNode::Text(text) => build_text(text, ctx),

        // Fallback for unimplemented nodes
        _ => {
            // Return a placeholder element
            // TODO: Implement remaining node types
            MathElement::HBox {
                children: vec![],
                width: 0.0,
                height: 0.0,
                depth: 0.0,
                classes: if ctx.config.include_classes {
                    vec![format!("unimplemented-{:?}", node.typ())]
                } else {
                    vec![]
                },
            }
        }
    }
}

// =============================================================================
// Text/Symbol Builders
// =============================================================================

fn build_math_ord(ord: &MathOrdNode, ctx: &LayoutContext) -> MathElement {
    build_symbol(&ord.text, ctx, true)
}

fn build_text_ord(ord: &TextOrdNode, ctx: &LayoutContext) -> MathElement {
    build_symbol(&ord.text, ctx, false)
}

fn build_atom(atom: &AtomNode, ctx: &LayoutContext) -> MathElement {
    build_symbol(&atom.text, ctx, ctx.mode() == Mode::Math)
}

/// Build a symbol with proper font metrics.
fn build_symbol(text: &str, ctx: &LayoutContext, math_mode: bool) -> MathElement {
    let mode = if math_mode { Mode::Math } else { Mode::Text };

    // Determine the font to use
    let font_name = determine_font_for_symbol(text, ctx, math_mode);

    // Get metrics for the first character
    let first_char = text.chars().next().unwrap_or('?');
    let metrics = ctx.char_metrics(first_char, font_name, mode);

    let (width, italic, skew) = if let Some(m) = metrics {
        (m.width, m.italic, m.skew)
    } else {
        // Fallback approximation
        (0.5, 0.0, 0.0)
    };

    let size = ctx.size_multiplier();

    MathElement::Text {
        text: text.to_string(),
        style: TextStyle {
            font: Some(Font::from_name(font_name)),
            size,
            color: ctx.options.get_color(),
            italic_correction: if math_mode { italic } else { 0.0 },
            skew,
            width: Some(width * size),
        },
    }
}

/// Determine which font to use for a symbol.
fn determine_font_for_symbol(text: &str, ctx: &LayoutContext, math_mode: bool) -> &'static str {
    let mode = if math_mode { Mode::Math } else { Mode::Text };

    // Check if there's a font override
    if !ctx.options.font.is_empty() {
        match ctx.options.font.as_ref() {
            "mathbf" => return "Main-Bold",
            "mathit" => return "Main-Italic",
            "mathrm" => return "Main-Regular",
            "mathsf" => return "SansSerif-Regular",
            "mathtt" => return "Typewriter-Regular",
            "mathcal" => return "Caligraphic-Regular",
            "mathfrak" => return "Fraktur-Regular",
            "mathscr" => return "Script-Regular",
            "boldsymbol" => return "Math-BoldItalic",
            _ => {}
        }
    }

    // For math mode, use Math-Italic for variables
    if math_mode {
        // Check if the symbol is defined in the symbol table
        if let Some(sym) = symbols::SYMBOLS.get(mode, text) {
            match sym.font {
                symbols::Font::Main => return "Main-Regular",
                symbols::Font::Ams => return "AMS-Regular",
            }
        }

        // Default to Math-Italic for math mode
        "Math-Italic"
    } else {
        // Text mode uses Main-Regular
        "Main-Regular"
    }
}

// =============================================================================
// Spacing Builders
// =============================================================================

fn build_spacing(sp: &SpacingNode, ctx: &LayoutContext) -> MathElement {
    // Get spacing amount from the spacing command
    let width = match sp.text.as_str() {
        "\\," | "\\thinspace" => 3.0 / 18.0,
        "\\:" | "\\medspace" => 4.0 / 18.0,
        "\\;" | "\\thickspace" => 5.0 / 18.0,
        "\\!" | "\\negthinspace" => -3.0 / 18.0,
        "\\negmedspace" => -4.0 / 18.0,
        "\\negthickspace" => -5.0 / 18.0,
        "\\ " | "~" => 0.5, // Non-breaking space
        "\\quad" => 1.0,
        "\\qquad" => 2.0,
        _ => 0.0,
    };

    MathElement::Kern {
        width: width * ctx.size_multiplier(),
    }
}

fn build_kern(kern: &KernNode, ctx: &LayoutContext) -> MathElement {
    let width = calculate_size(&kern.dimension, ctx.options);
    MathElement::Kern { width }
}

// =============================================================================
// Fraction Builder
// =============================================================================

fn build_fraction(frac: &GenFracNode, ctx: &LayoutContext) -> MathElement {
    let metrics = ctx.metrics();

    // Build numerator and denominator
    // TODO: Use proper fraction styles (frac_num, frac_den)
    let numer = build_node(&frac.numer, ctx);
    let denom = build_node(&frac.denom, ctx);

    let numer_width = numer.width();
    let denom_width = denom.width();
    let width = numer_width.max(denom_width);

    // Compute vertical positioning (TeX Rule 15)
    let is_display = ctx.is_display();
    let axis_height = metrics.axis_height;

    let (num_shift, denom_shift, rule_width) = if frac.has_bar_line {
        let rule_width = frac.bar_size
            .as_ref()
            .map(|m| calculate_size(m, ctx.options))
            .unwrap_or(metrics.default_rule_thickness);

        let num_shift = if is_display {
            metrics.num1
        } else {
            metrics.num2
        };
        let denom_shift = if is_display {
            metrics.denom1
        } else {
            metrics.denom2
        };

        (num_shift, denom_shift, rule_width)
    } else {
        // No bar (like \atop or \binom)
        let num_shift = if is_display {
            metrics.num1
        } else {
            metrics.num3
        };
        let denom_shift = if is_display {
            metrics.denom1
        } else {
            metrics.denom2
        };

        (num_shift, denom_shift, 0.0)
    };

    // Position numerator and denominator
    let numer_x = (width - numer_width) / 2.0;
    let denom_x = (width - denom_width) / 2.0;

    let numer_pos = Positioned::new(numer.clone(), numer_x, num_shift);
    let denom_pos = Positioned::new(denom.clone(), denom_x, -denom_shift);

    // Build the layout
    let mut children = vec![numer_pos.clone(), denom_pos.clone()];

    // Add rule if needed
    let bar = if frac.has_bar_line && rule_width > 0.0 {
        let rule = MathElement::Rule {
            width,
            height: rule_width,
            shift: axis_height,
            style: LineStyle::Solid,
            color: None,
        };
        children.push(Positioned::new(rule, 0.0, axis_height));

        Some(FractionBar {
            thickness: rule_width,
            color: None,
            style: LineStyle::Solid,
        })
    } else {
        None
    };

    let height = num_shift + numer.height();
    let depth = denom_shift + denom.depth();

    let layout = MathElement::VBox {
        children: children.clone(),
        width,
        height,
        depth,
    };

    if ctx.config.semantic_mode {
        MathElement::Fraction {
            numerator: Box::new(numer_pos),
            denominator: Box::new(denom_pos),
            bar,
            layout: Box::new(layout),
        }
    } else {
        layout
    }
}

// =============================================================================
// Superscript/Subscript Builder
// =============================================================================

fn build_supsub(supsub: &SupSubNode, ctx: &LayoutContext) -> MathElement {
    let metrics = ctx.metrics();

    // Build base
    let base = supsub.base.as_ref().map(|b| build_node(b, ctx));
    let base_height = base.as_ref().map(|b| b.height()).unwrap_or(0.0);
    let base_depth = base.as_ref().map(|b| b.depth()).unwrap_or(0.0);
    let base_width = base.as_ref().map(|b| b.width()).unwrap_or(0.0);

    // Build scripts (TODO: use proper script style)
    let sup = supsub.sup.as_ref().map(|s| build_node(s, ctx));
    let sub = supsub.sub.as_ref().map(|s| build_node(s, ctx));

    // Compute positions using TeX rules (Rule 18)
    // These are the font metric values for script positioning
    let mut sup_shift = metrics.sup1; // Default superscript shift
    let mut sub_shift = metrics.sub1; // Default subscript shift

    // Adjust based on style
    if ctx.is_cramped() {
        sup_shift = metrics.sup3;
    }

    // Ensure minimum clearance
    if let Some(ref sup_elem) = sup {
        let min_sup_shift = base_height - sup_elem.depth() + metrics.x_height / 4.0;
        sup_shift = sup_shift.max(min_sup_shift);
    }

    if let Some(ref sub_elem) = sub {
        let min_sub_shift = sub_elem.height() - base_depth * 4.0 / 5.0;
        sub_shift = sub_shift.max(min_sub_shift);
    }

    // If both sup and sub, adjust to prevent overlap
    if sup.is_some() && sub.is_some() {
        let sup_elem = sup.as_ref().unwrap();
        let sub_elem = sub.as_ref().unwrap();

        let gap = (sup_shift - sup_elem.depth()) - (sub_elem.height() - sub_shift);
        let min_gap = 4.0 * metrics.default_rule_thickness;

        if gap < min_gap {
            let adjustment = (min_gap - gap) / 2.0;
            sup_shift += adjustment;
            sub_shift += adjustment;
        }
    }

    // Build positioned elements
    let base_pos = base.map(|b| Positioned::new(b, 0.0, 0.0));

    let sup_pos = sup.map(|s| {
        Positioned::new(s, base_width, sup_shift)
    });

    let sub_pos = sub.map(|s| {
        Positioned::new(s, base_width, -sub_shift)
    });

    // Compute overall dimensions
    let script_width = sup_pos.as_ref().map(|s| s.element.width()).unwrap_or(0.0)
        .max(sub_pos.as_ref().map(|s| s.element.width()).unwrap_or(0.0));
    let total_width = base_width + script_width;

    let height = base_height.max(
        sup_pos.as_ref().map(|s| s.y + s.element.height()).unwrap_or(0.0)
    );
    let depth = base_depth.max(
        sub_pos.as_ref().map(|s| -s.y + s.element.depth()).unwrap_or(0.0)
    );

    // Build layout HBox
    let mut children = Vec::new();
    if let Some(b) = base_pos.clone() {
        children.push(b);
    }
    if let Some(s) = sup_pos.clone() {
        children.push(s);
    }
    if let Some(s) = sub_pos.clone() {
        children.push(s);
    }

    let layout = MathElement::HBox {
        children,
        width: total_width,
        height,
        depth,
        classes: if ctx.config.include_classes {
            vec!["mord".to_string(), "supsub".to_string()]
        } else {
            vec![]
        },
    };

    if ctx.config.semantic_mode {
        MathElement::Scripts {
            base: base_pos.map(Box::new),
            superscript: sup_pos.map(Box::new),
            subscript: sub_pos.map(Box::new),
            layout: Box::new(layout),
        }
    } else {
        layout
    }
}

// =============================================================================
// Placeholder Builders (to be implemented)
// =============================================================================

fn build_sqrt(sqrt: &SqrtNode, ctx: &LayoutContext) -> MathElement {
    // TODO: Implement proper sqrt layout with surd and vinculum
    let radicand = build_node(&sqrt.body, ctx);
    let index = sqrt.index.as_ref().map(|i| build_node(i, ctx));

    let (width, height, depth) = radicand.dimensions();

    // Simple placeholder layout
    let layout = MathElement::HBox {
        children: vec![Positioned::at_origin(radicand.clone())],
        width: width + 0.5, // Add space for surd
        height: height + 0.1, // Add space for vinculum
        depth,
        classes: if ctx.config.include_classes {
            vec!["sqrt".to_string()]
        } else {
            vec![]
        },
    };

    if ctx.config.semantic_mode {
        MathElement::Radical {
            radicand: Box::new(radicand),
            index: index.map(Box::new),
            layout: Box::new(layout),
        }
    } else {
        layout
    }
}

fn build_accent(accent: &AccentNode, ctx: &LayoutContext) -> MathElement {
    let base = build_node(&accent.base, ctx);
    let accent_sym = build_symbol(&accent.label, ctx, true);

    let (base_width, base_height, base_depth) = base.dimensions();
    let (_, accent_height, _) = accent_sym.dimensions();

    let layout = MathElement::VBox {
        children: vec![
            Positioned::new(accent_sym.clone(), 0.0, base_height),
            Positioned::at_origin(base.clone()),
        ],
        width: base_width,
        height: base_height + accent_height,
        depth: base_depth,
    };

    if ctx.config.semantic_mode {
        MathElement::Accent {
            base: Box::new(base),
            accent: Box::new(accent_sym),
            is_over: true,
            layout: Box::new(layout),
        }
    } else {
        layout
    }
}

fn build_op(op: &OpNode, ctx: &LayoutContext) -> MathElement {
    // Build the operator symbol
    let name = op.name.as_deref().unwrap_or("?");
    build_symbol(name, ctx, true)
}

fn build_color(color: &ColorNode, ctx: &LayoutContext) -> MathElement {
    let inner = build_expression(&color.body, ctx);
    MathElement::Color {
        color: color.color.clone(),
        inner: Box::new(inner),
    }
}

fn build_font(font: &FontNode, ctx: &LayoutContext) -> MathElement {
    // TODO: Apply font to context
    build_node(&font.body, ctx)
}

fn build_styling(styling: &StylingNode, ctx: &LayoutContext) -> MathElement {
    // TODO: Apply style to context
    build_expression(&styling.body, ctx)
}

fn build_sizing(sizing: &SizingNode, ctx: &LayoutContext) -> MathElement {
    // TODO: Apply size to context
    build_expression(&sizing.body, ctx)
}

fn build_overline(over: &OverlineNode, ctx: &LayoutContext) -> MathElement {
    let base = build_node(&over.body, ctx);
    let (width, height, depth) = base.dimensions();

    let rule_thickness = ctx.metrics().default_rule_thickness;

    let rule = MathElement::Rule {
        width,
        height: rule_thickness,
        shift: height + rule_thickness,
        style: LineStyle::Solid,
        color: None,
    };

    let layout = MathElement::VBox {
        children: vec![
            Positioned::new(rule.clone(), 0.0, height + rule_thickness),
            Positioned::at_origin(base.clone()),
        ],
        width,
        height: height + 2.0 * rule_thickness,
        depth,
    };

    if ctx.config.semantic_mode {
        MathElement::Accent {
            base: Box::new(base),
            accent: Box::new(rule),
            is_over: true,
            layout: Box::new(layout),
        }
    } else {
        layout
    }
}

fn build_underline(under: &UnderlineNode, ctx: &LayoutContext) -> MathElement {
    let base = build_node(&under.body, ctx);
    let (width, height, depth) = base.dimensions();

    let rule_thickness = ctx.metrics().default_rule_thickness;

    let rule = MathElement::Rule {
        width,
        height: rule_thickness,
        shift: -(depth + rule_thickness),
        style: LineStyle::Solid,
        color: None,
    };

    let layout = MathElement::VBox {
        children: vec![
            Positioned::at_origin(base.clone()),
            Positioned::new(rule.clone(), 0.0, -(depth + rule_thickness)),
        ],
        width,
        height,
        depth: depth + 2.0 * rule_thickness,
    };

    if ctx.config.semantic_mode {
        MathElement::Accent {
            base: Box::new(base),
            accent: Box::new(rule),
            is_over: false,
            layout: Box::new(layout),
        }
    } else {
        layout
    }
}

fn build_phantom(phantom: &PhantomNode, ctx: &LayoutContext) -> MathElement {
    let inner = build_expression(&phantom.body, ctx);
    MathElement::Phantom {
        inner: Box::new(inner),
    }
}

fn build_hphantom(hphantom: &HPhantomNode, ctx: &LayoutContext) -> MathElement {
    let inner = build_node(&hphantom.body, ctx);
    let width = inner.width();
    MathElement::Kern { width }
}

fn build_vphantom(vphantom: &VPhantomNode, ctx: &LayoutContext) -> MathElement {
    let inner = build_node(&vphantom.body, ctx);
    let (_, height, depth) = inner.dimensions();

    // Zero-width element with the height/depth of the inner
    MathElement::HBox {
        children: vec![],
        width: 0.0,
        height,
        depth,
        classes: vec![],
    }
}

fn build_rule(rule: &RuleNode, ctx: &LayoutContext) -> MathElement {
    let width = calculate_size(&rule.width, ctx.options);
    let height = calculate_size(&rule.height, ctx.options);
    let shift = rule.shift.as_ref()
        .map(|s| calculate_size(s, ctx.options))
        .unwrap_or(0.0);

    MathElement::Rule {
        width,
        height,
        shift,
        style: LineStyle::Solid,
        color: None,
    }
}

fn build_left_right(lr: &LeftRightNode, ctx: &LayoutContext) -> MathElement {
    let body = build_expression(&lr.body, ctx);
    let (_, height, depth) = body.dimensions();

    // TODO: Build proper stretchy delimiters
    // left and right are Strings; "." means no delimiter
    let left = if lr.left.is_empty() || lr.left == "." {
        None
    } else {
        Some(build_symbol(&lr.left, ctx, true))
    };
    let right = if lr.right.is_empty() || lr.right == "." {
        None
    } else {
        Some(build_symbol(&lr.right, ctx, true))
    };

    let left_width = left.as_ref().map(|l| l.width()).unwrap_or(0.0);
    let right_width = right.as_ref().map(|r| r.width()).unwrap_or(0.0);
    let body_width = body.width();

    let mut children = Vec::new();
    let mut x = 0.0;

    if let Some(ref l) = left {
        children.push(Positioned::new(l.clone(), x, 0.0));
        x += left_width;
    }

    children.push(Positioned::new(body.clone(), x, 0.0));
    x += body_width;

    if let Some(ref r) = right {
        children.push(Positioned::new(r.clone(), x, 0.0));
        x += right_width;
    }

    let layout = MathElement::HBox {
        children,
        width: x,
        height,
        depth,
        classes: vec![],
    };

    if ctx.config.semantic_mode {
        MathElement::Delimited {
            left: left.map(Box::new),
            right: right.map(Box::new),
            body: Box::new(body),
            layout: Box::new(layout),
        }
    } else {
        layout
    }
}

fn build_text(text: &TextNode, ctx: &LayoutContext) -> MathElement {
    // Build text mode content
    build_expression(&text.body, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParserConfig;

    fn default_options() -> Options {
        Options::from_parser_conf(&ParserConfig::default())
    }

    #[test]
    fn test_build_simple_symbol() {
        let opts = default_options();
        let config = IrBuilderConfig::default();
        let ctx = LayoutContext::new(&opts, &config);

        let elem = build_symbol("x", &ctx, true);

        if let MathElement::Text { text, style } = elem {
            assert_eq!(text, "x");
            assert!(style.width.is_some());
            assert!(style.width.unwrap() > 0.0);
        } else {
            panic!("Expected Text element");
        }
    }

    #[test]
    fn test_build_empty_expression() {
        let opts = default_options();
        let layout = build_ir(&[], &opts);

        assert_eq!(layout.width, 0.0);
        assert_eq!(layout.height, 0.0);
    }

    #[test]
    fn test_semantic_mode_fraction() {
        let opts = default_options();

        // With semantic mode
        let semantic_config = IrBuilderConfig::default();
        let ctx = LayoutContext::new(&opts, &semantic_config);

        let frac_node = GenFracNode {
            continued: false,
            numer: Box::new(ParseNode::MathOrd(MathOrdNode {
                text: "1".to_string(),
                info: NodeInfo::new_mode(Mode::Math),
            })),
            denom: Box::new(ParseNode::MathOrd(MathOrdNode {
                text: "2".to_string(),
                info: NodeInfo::new_mode(Mode::Math),
            })),
            has_bar_line: true,
            left_delim: None,
            right_delim: None,
            size: crate::util::StyleAuto::Auto,
            bar_size: None,
            info: NodeInfo::new_mode(Mode::Math),
        };

        let elem = build_fraction(&frac_node, &ctx);
        assert!(matches!(elem, MathElement::Fraction { .. }));

        // Without semantic mode
        let layout_config = IrBuilderConfig::layout_only();
        let ctx = LayoutContext::new(&opts, &layout_config);

        let elem = build_fraction(&frac_node, &ctx);
        assert!(matches!(elem, MathElement::VBox { .. }));
    }
}
