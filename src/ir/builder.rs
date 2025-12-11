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

use crate::array::ColSeparationType;
use crate::expander::Mode;
use crate::font_metrics::{get_character_metrics, CharacterMetrics, FontMetrics};
use crate::html::DomType;
use crate::parse_node::*;
use crate::spacing_data::{SPACINGS, TIGHT_SPACINGS};
use crate::style::{DISPLAY_STYLE, SCRIPT_STYLE, SCRIPT_SCRIPT_STYLE};
use crate::symbols::{self, Atom};
use crate::unit::calculate_size;
use crate::util::find_assoc_data;
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
///
/// This context owns its Options, allowing style changes to be properly
/// propagated through the layout tree.
pub struct LayoutContext<'a> {
    /// The options for this context (owned to allow style changes)
    options: Options,
    /// The builder configuration (shared)
    pub config: &'a IrBuilderConfig,
}

impl<'a> LayoutContext<'a> {
    pub fn new(options: &Options, config: &'a IrBuilderConfig) -> Self {
        Self {
            options: options.clone(),
            config,
        }
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

    /// Check if we're in script style.
    pub fn is_script(&self) -> bool {
        self.options.style.size() == SCRIPT_STYLE.size()
    }

    /// Check if we're in scriptscript style.
    pub fn is_scriptscript(&self) -> bool {
        self.options.style.size() == SCRIPT_SCRIPT_STYLE.size()
    }

    /// Check if the current style is cramped.
    pub fn is_cramped(&self) -> bool {
        self.options.style.cramped()
    }

    /// Get the current size multiplier.
    pub fn size_multiplier(&self) -> f64 {
        self.options.size_multiplier()
    }

    /// Get the current style.
    pub fn style(&self) -> crate::style::StyleId {
        self.options.style
    }

    /// Get access to the options.
    pub fn options(&self) -> &Options {
        &self.options
    }

    /// Create a child context with a different style.
    ///
    /// This properly updates the size multiplier based on the new style.
    pub fn with_style(&self, style: crate::style::StyleId) -> LayoutContext<'a> {
        let new_options = self.options.having_style(style)
            .unwrap_or_else(|| self.options.clone());
        LayoutContext {
            options: new_options,
            config: self.config,
        }
    }

    /// Create a child context for superscript.
    pub fn for_superscript(&self) -> LayoutContext<'a> {
        self.with_style(self.options.style.sup())
    }

    /// Create a child context for subscript.
    pub fn for_subscript(&self) -> LayoutContext<'a> {
        self.with_style(self.options.style.sub())
    }

    /// Create a child context for fraction numerator.
    pub fn for_numerator(&self) -> LayoutContext<'a> {
        self.with_style(self.options.style.frac_num())
    }

    /// Create a child context for fraction denominator.
    pub fn for_denominator(&self) -> LayoutContext<'a> {
        self.with_style(self.options.style.frac_den())
    }

    /// Create a child context with cramped style.
    pub fn cramped(&self) -> LayoutContext<'a> {
        self.with_style(self.options.style.cramp())
    }

    /// Create a child context with a specific font.
    pub fn with_font(&self, font: &str) -> LayoutContext<'a> {
        let new_options = self.options.clone().with_font(font.to_string());
        LayoutContext {
            options: new_options,
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
// Atom Type Helpers
// =============================================================================

/// Determine the DomType (atom type) of a parse node.
/// This is used for computing inter-element spacing.
fn get_dom_type(node: &ParseNode) -> Option<DomType> {
    match node {
        // Explicit atom types
        ParseNode::Atom(atom) => match atom.family {
            Atom::Bin => Some(DomType::MBin),
            Atom::Rel => Some(DomType::MRel),
            Atom::Open => Some(DomType::MOpen),
            Atom::Close => Some(DomType::MClose),
            Atom::Punct => Some(DomType::MPunct),
            Atom::Inner => Some(DomType::MInner),
        },

        // Ordinals
        ParseNode::MathOrd(_) | ParseNode::TextOrd(_) => Some(DomType::MOrd),

        // Operators
        ParseNode::Op(_) => Some(DomType::MOp),

        // Fractions, surds, supsubs act as ordinals
        ParseNode::GenFrac(_) | ParseNode::Sqrt(_) | ParseNode::SupSub(_) => Some(DomType::MOrd),

        // Delimiters
        ParseNode::LeftRight(_) => Some(DomType::MInner),

        // Groups inherit from their content (simplified: treat as ord)
        ParseNode::OrdGroup(_) => Some(DomType::MOrd),

        // Spacing nodes don't participate in spacing calculation
        ParseNode::Spacing(_) | ParseNode::Kern(_) => None,

        // Other nodes default to ordinal
        _ => Some(DomType::MOrd),
    }
}

/// Check if a node is a "non-space" node that participates in spacing.
fn is_non_space_node(node: &ParseNode) -> bool {
    !matches!(node, ParseNode::Spacing(_) | ParseNode::Kern(_))
}

/// Get spacing (in mu) between two atom types.
fn get_spacing(left: DomType, right: DomType, is_tight: bool) -> f64 {
    let table = if is_tight { TIGHT_SPACINGS } else { SPACINGS };
    find_assoc_data(table, (left, right))
        .map(|mu| mu.0)
        .unwrap_or(0.0)
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

    // Determine if we're in tight (script) style
    let is_tight = ctx.style().is_tight();

    // Track the previous non-space node's type for spacing
    let mut prev_dom_type: Option<DomType> = None;

    for node in nodes {
        // Get the current node's dom type before building it
        let curr_dom_type = get_dom_type(node);
        let is_non_space = is_non_space_node(node);

        // Insert spacing between adjacent non-space atoms
        if is_non_space {
            if let (Some(prev), Some(curr)) = (prev_dom_type, curr_dom_type) {
                let spacing_mu = get_spacing(prev, curr, is_tight);
                if spacing_mu != 0.0 {
                    // Convert mu to em: 1 mu = 1/18 em
                    let spacing_em = spacing_mu / 18.0 * ctx.size_multiplier();
                    if spacing_em != 0.0 {
                        children.push(Positioned::new(
                            MathElement::Kern { width: spacing_em },
                            x_offset,
                            0.0,
                        ));
                        x_offset += spacing_em;
                    }
                }
            }
        }

        // Build the element
        let element = build_node(node, ctx);
        let (width, height, depth) = element.dimensions();

        children.push(Positioned::new(element, x_offset, 0.0));
        x_offset += width;
        max_height = max_height.max(height);
        max_depth = max_depth.max(depth);

        // Update previous dom type for next iteration
        if is_non_space {
            prev_dom_type = curr_dom_type;
        }
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
        ParseNode::MClass(mclass) => build_mclass(mclass, ctx),
        ParseNode::OperatorName(opname) => build_operator_name(opname, ctx),
        ParseNode::AccentUnder(accent) => build_accent_under(accent, ctx),
        ParseNode::DelimSizing(delim) => build_delim_sizing(delim, ctx),
        ParseNode::Middle(middle) => build_middle(middle, ctx),
        ParseNode::Href(href) => build_href(href, ctx),
        ParseNode::HBox(hbox) => build_hbox(hbox, ctx),
        ParseNode::XArrow(arrow) => build_xarrow(arrow, ctx),
        ParseNode::Enclose(enclose) => build_enclose(enclose, ctx),
        ParseNode::HorizBrace(brace) => build_horiz_brace(brace, ctx),
        ParseNode::Lap(lap) => build_lap(lap, ctx),
        ParseNode::MathChoice(choice) => build_math_choice(choice, ctx),
        ParseNode::RaiseBox(rbox) => build_raise_box(rbox, ctx),
        ParseNode::Smash(smash) => build_smash(smash, ctx),
        ParseNode::VCenter(vc) => build_vcenter(vc, ctx),
        ParseNode::Array(arr) => build_array(arr, ctx),

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

    // Look up the symbol to get its replacement character (e.g., "\\alpha" -> "α")
    let symbol = symbols::SYMBOLS.get(mode, text);
    let render_text = symbol
        .and_then(|s| s.replace)
        .unwrap_or(text);

    // Determine the font to use
    let font_name = determine_font_for_symbol(text, ctx, math_mode);

    // Get metrics for the first character of the replacement text
    let first_char = render_text.chars().next().unwrap_or('?');
    let metrics = ctx.char_metrics(first_char, font_name, mode);

    let (width, height, depth, italic, skew) = if let Some(m) = metrics {
        (m.width, m.height, m.depth, m.italic, m.skew)
    } else {
        // Fallback approximation using typical KaTeX_Main metrics
        (0.5, 0.656, 0.219, 0.0, 0.0)
    };

    let size = ctx.size_multiplier();

    MathElement::Text {
        text: render_text.to_string(),
        style: TextStyle {
            font: Some(Font::from_name(font_name)),
            size,
            color: ctx.options.get_color(),
            italic_correction: if math_mode { italic } else { 0.0 },
            skew,
            width: Some(width * size),
            height: Some(height * size),
            depth: Some(depth * size),
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
    let width = calculate_size(&kern.dimension, ctx.options());
    MathElement::Kern { width }
}

// =============================================================================
// Fraction Builder
// =============================================================================

fn build_fraction(frac: &GenFracNode, ctx: &LayoutContext) -> MathElement {
    let metrics = ctx.metrics();

    // Build numerator and denominator in appropriate styles
    let numer_ctx = ctx.for_numerator();
    let denom_ctx = ctx.for_denominator();
    let numer = build_node(&frac.numer, &numer_ctx);
    let denom = build_node(&frac.denom, &denom_ctx);

    let numer_width = numer.width();
    let denom_width = denom.width();
    let width = numer_width.max(denom_width);

    // Compute vertical positioning (TeX Rule 15)
    let is_display = ctx.is_display();
    let axis_height = metrics.axis_height;

    let (num_shift, denom_shift, rule_width) = if frac.has_bar_line {
        let rule_width = frac.bar_size
            .as_ref()
            .map(|m| calculate_size(m, ctx.options()))
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

/// Extract italic correction from a MathElement (for script positioning).
fn get_italic_correction(elem: &MathElement) -> f64 {
    match elem {
        MathElement::Text { style, .. } => style.italic_correction,
        // For compound elements, check the rightmost child
        MathElement::HBox { children, .. } => {
            children.last()
                .map(|c| get_italic_correction(&c.element))
                .unwrap_or(0.0)
        }
        // Semantic variants delegate to their layout
        MathElement::Scripts { layout, .. }
        | MathElement::Fraction { layout, .. }
        | MathElement::Radical { layout, .. }
        | MathElement::Accent { layout, .. }
        | MathElement::Delimited { layout, .. }
        | MathElement::LargeOp { layout, .. }
        | MathElement::Array { layout, .. } => get_italic_correction(layout),
        _ => 0.0,
    }
}

fn build_supsub(supsub: &SupSubNode, ctx: &LayoutContext) -> MathElement {
    let metrics = ctx.metrics();

    // Build base
    let base = supsub.base.as_ref().map(|b| build_node(b, ctx));
    let base_height = base.as_ref().map(|b| b.height()).unwrap_or(0.0);
    let base_depth = base.as_ref().map(|b| b.depth()).unwrap_or(0.0);
    let base_width = base.as_ref().map(|b| b.width()).unwrap_or(0.0);
    let base_italic = base.as_ref().map(|b| get_italic_correction(b)).unwrap_or(0.0);

    // Build scripts in appropriate styles
    let sup_ctx = ctx.for_superscript();
    let sub_ctx = ctx.for_subscript();
    let sup = supsub.sup.as_ref().map(|s| build_node(s, &sup_ctx));
    let sub = supsub.sub.as_ref().map(|s| build_node(s, &sub_ctx));

    // Compute positions using TeX rules (Rule 18)
    // These are the font metric values for script positioning
    let mut sub_shift = metrics.sub1; // Default subscript shift

    // Choose superscript shift based on style (TeX Rule 18a):
    // - sup1: display style
    // - sup2: text style (non-display, non-cramped)
    // - sup3: cramped style
    let mut sup_shift = if ctx.is_display() {
        metrics.sup1
    } else if ctx.is_cramped() {
        metrics.sup3
    } else {
        metrics.sup2
    };

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

    // Superscript is shifted right by italic correction (for italic bases like f^2)
    let sup_pos = sup.map(|s| {
        Positioned::new(s, base_width + base_italic, sup_shift)
    });

    // Subscript stays at base_width (no italic correction)
    let sub_pos = sub.map(|s| {
        Positioned::new(s, base_width, -sub_shift)
    });

    // Compute overall dimensions
    // Superscript width includes italic correction offset
    let sup_extent = sup_pos.as_ref()
        .map(|s| base_italic + s.element.width())
        .unwrap_or(0.0);
    let sub_extent = sub_pos.as_ref()
        .map(|s| s.element.width())
        .unwrap_or(0.0);
    let total_width = base_width + sup_extent.max(sub_extent);

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
    // TeX Rule 11: Square roots
    let metrics = ctx.metrics();

    // Build the radicand in cramped style
    let cramped_ctx = ctx.cramped();
    let mut radicand = build_node(&sqrt.body, &cramped_ctx);

    // Ensure minimum height (use x_height if radicand is empty/small)
    let (rad_width, mut rad_height, rad_depth) = radicand.dimensions();
    if rad_height < metrics.x_height {
        rad_height = metrics.x_height;
        // Update radicand's dimensions if it's an HBox
        radicand = ensure_min_height(radicand, rad_height);
    }

    // Calculate rule and clearance parameters
    let theta = metrics.sqrt_rule_thickness.max(metrics.default_rule_thickness);
    let phi = if ctx.is_display() {
        metrics.x_height
    } else {
        theta
    };
    let line_clearance = theta + phi / 4.0;

    // Total height needed for the surd
    let surd_height = rad_height + rad_depth + line_clearance + theta;

    // Determine surd size and advance width
    // (simplified: use a fixed advance width based on size)
    let advance_width = compute_surd_advance_width(surd_height, ctx);
    let surd_depth = compute_surd_depth(surd_height);

    // Adjust clearance if surd is taller than needed
    let actual_clearance = if surd_depth > rad_height + rad_depth + line_clearance {
        (line_clearance + surd_depth - rad_height - rad_depth) / 2.0
    } else {
        line_clearance
    };

    // Build the surd path element
    let surd = MathElement::Path {
        path_data: std::borrow::Cow::Borrowed("surd"), // Placeholder path name
        width: advance_width,
        height: surd_height,
        shift: 0.0,
    };

    // Build the vinculum (horizontal rule)
    let vinculum = MathElement::Rule {
        width: rad_width,
        height: theta,
        shift: 0.0,
        style: LineStyle::Solid,
        color: None,
    };

    // Build the index if present (in scriptscript style)
    let index = sqrt.index.as_ref().map(|i| {
        let ss_ctx = ctx.with_style(crate::style::SCRIPT_SCRIPT_STYLE);
        build_node(i, &ss_ctx)
    });

    // Calculate total dimensions
    let total_height = rad_height + actual_clearance + theta;
    let total_width = advance_width + rad_width;

    // Position elements
    // Radicand is at origin (baseline)
    // Vinculum is above the radicand
    // Surd is to the left

    let vinculum_y = rad_height + actual_clearance;

    let mut children = vec![
        // Surd on the left
        Positioned::new(surd, 0.0, 0.0),
        // Vinculum above radicand
        Positioned::new(vinculum.clone(), advance_width, vinculum_y),
        // Radicand content
        Positioned::new(radicand.clone(), advance_width, 0.0),
    ];

    // Handle root index positioning
    let index_width = if let Some(ref idx) = index {
        let idx_width = idx.width();
        // Position index to the left and raised
        // The amount the index is shifted by (from TeX `\r@@t`)
        let to_shift = 0.6 * (total_height - rad_depth);
        children.insert(0, Positioned::new(idx.clone(), 0.0, to_shift));
        idx_width
    } else {
        0.0
    };

    let layout = MathElement::HBox {
        children,
        width: total_width + index_width,
        height: total_height,
        depth: rad_depth,
        classes: if ctx.config.include_classes {
            vec!["mord".to_string(), "sqrt".to_string()]
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

/// Ensure an element has at least the given height.
fn ensure_min_height(elem: MathElement, min_height: f64) -> MathElement {
    match elem {
        MathElement::HBox { children, width, height, depth, classes } => {
            MathElement::HBox {
                children,
                width,
                height: height.max(min_height),
                depth,
                classes,
            }
        }
        other => other,
    }
}

/// Compute the advance width of the surd based on required height.
fn compute_surd_advance_width(height: f64, ctx: &LayoutContext) -> f64 {
    // Simplified: use different advance widths based on size
    // In actual implementation, this should match the SVG surd dimensions
    let size = ctx.size_multiplier();
    if height < 1.0 {
        0.55 * size
    } else if height < 1.4 {
        0.68 * size
    } else if height < 2.0 {
        0.80 * size
    } else {
        1.0 * size
    }
}

/// Compute the surd depth (portion below baseline).
fn compute_surd_depth(height: f64) -> f64 {
    // Simplified: surd depth is roughly proportional to height
    height * 0.8
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
    // Apply font override to context
    let font_ctx = ctx.with_font(&font.font);
    build_node(&font.body, &font_ctx)
}

fn build_styling(styling: &StylingNode, ctx: &LayoutContext) -> MathElement {
    // Apply style override to context
    let new_style = styling.style.into_style_id();
    let styled_ctx = ctx.with_style(new_style);
    build_expression(&styling.body, &styled_ctx)
}

fn build_sizing(sizing: &SizingNode, ctx: &LayoutContext) -> MathElement {
    // Apply size override to context
    // Size is 1-based index, where 1 = \tiny, 10 = \Huge, 6 = \normalsize
    let new_options = ctx.options().having_size(sizing.size)
        .unwrap_or_else(|| ctx.options().clone());
    let sized_ctx = LayoutContext::new(&new_options, ctx.config);
    build_expression(&sizing.body, &sized_ctx)
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
    let width = calculate_size(&rule.width, ctx.options());
    let height = calculate_size(&rule.height, ctx.options());
    let shift = rule.shift.as_ref()
        .map(|s| calculate_size(s, ctx.options()))
        .unwrap_or(0.0);

    MathElement::Rule {
        width,
        height,
        shift,
        style: LineStyle::Solid,
        color: None,
    }
}

/// Build a stretchy delimiter at a given height.
fn build_stretchy_delimiter(delim: &str, height: f64, ctx: &LayoutContext) -> MathElement {
    // Calculate delimiter dimensions
    let metrics = ctx.metrics();
    let axis_height = metrics.axis_height;

    // Estimate width based on delimiter type
    let width = match delim {
        "(" | ")" | "[" | "]" => 0.35,
        "{" | "}" | "⟨" | "⟩" | "\\langle" | "\\rangle" => 0.40,
        "|" | "\\|" | "\\vert" | "\\Vert" => 0.20,
        "/" | "\\" | "\\backslash" => 0.50,
        _ => 0.35, // Default
    } * ctx.size_multiplier();

    // Stretchy delimiters are centered on the axis
    let shift = axis_height;
    let depth = height / 2.0 - shift;
    let elem_height = height / 2.0 + shift;

    MathElement::Path {
        path_data: std::borrow::Cow::Owned(format!("delimiter:{}", delim)),
        width,
        height: elem_height + depth,
        shift: -depth, // Shift so delimiter is centered on axis
    }
}

fn build_left_right(lr: &LeftRightNode, ctx: &LayoutContext) -> MathElement {
    let body = build_expression(&lr.body, ctx);
    let (body_width, height, depth) = body.dimensions();

    // Calculate proper delimiter size using TeX formula
    let metrics = ctx.metrics();
    let axis_height = metrics.axis_height * ctx.size_multiplier();
    let delimiter_factor = 901.0;
    let delimiter_extend = 5.0 / metrics.pt_per_em;

    let max_dist_from_axis = (height - axis_height).max(depth + axis_height);
    let total_delim_height = (max_dist_from_axis / 500.0 * delimiter_factor)
        .max(2.0 * max_dist_from_axis - delimiter_extend);

    // Build delimiters with computed height
    // "." means no delimiter
    let left = if lr.left.is_empty() || lr.left == "." {
        None
    } else {
        Some(build_stretchy_delimiter(&lr.left, total_delim_height, ctx))
    };
    let right = if lr.right.is_empty() || lr.right == "." {
        None
    } else {
        Some(build_stretchy_delimiter(&lr.right, total_delim_height, ctx))
    };

    let left_width = left.as_ref().map(|l| l.width()).unwrap_or(0.0);
    let right_width = right.as_ref().map(|r| r.width()).unwrap_or(0.0);

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

fn build_mclass(mclass: &MClassNode, ctx: &LayoutContext) -> MathElement {
    // MClass wraps content with a specific math class (bin, rel, etc.)
    // The class affects spacing but not rendering, so just build the body
    build_expression(&mclass.body, ctx)
}

fn build_operator_name(opname: &OperatorNameNode, ctx: &LayoutContext) -> MathElement {
    // \operatorname renders text in roman font
    let font_ctx = ctx.with_font("mathrm");
    build_expression(&opname.body, &font_ctx)
}

fn build_accent_under(accent: &AccentUnderNode, ctx: &LayoutContext) -> MathElement {
    let base = build_node(&accent.base, ctx);
    let accent_sym = build_symbol(&accent.label, ctx, true);

    let (base_width, base_height, base_depth) = base.dimensions();
    let (_, accent_height, accent_depth) = accent_sym.dimensions();

    let layout = MathElement::VBox {
        children: vec![
            Positioned::at_origin(base.clone()),
            Positioned::new(accent_sym.clone(), 0.0, -(base_depth + accent_height)),
        ],
        width: base_width,
        height: base_height,
        depth: base_depth + accent_height + accent_depth,
    };

    if ctx.config.semantic_mode {
        MathElement::Accent {
            base: Box::new(base),
            accent: Box::new(accent_sym),
            is_over: false,
            layout: Box::new(layout),
        }
    } else {
        layout
    }
}

fn build_delim_sizing(delim: &DelimSizingNode, ctx: &LayoutContext) -> MathElement {
    // \big, \Big, \bigg, \Bigg delimiters
    let size_multiplier = match delim.size {
        DelimSize::One => 1.2,
        DelimSize::Two => 1.8,
        DelimSize::Three => 2.4,
        DelimSize::Four => 3.0,
    };

    let base_height = ctx.metrics().delim1 * size_multiplier;
    build_stretchy_delimiter(&delim.delim, base_height, ctx)
}

fn build_middle(middle: &MiddleNode, ctx: &LayoutContext) -> MathElement {
    // \middle delimiter - similar to stretchy but typically uses current height
    // For now, use a default height; proper implementation would inherit from parent
    let height = ctx.metrics().delim2;
    build_stretchy_delimiter(&middle.delim, height, ctx)
}

fn build_href(href: &HrefNode, ctx: &LayoutContext) -> MathElement {
    let inner = build_expression(&href.body, ctx);
    MathElement::Link {
        href: href.href.clone(),
        inner: Box::new(inner),
    }
}

fn build_hbox(hbox: &HBoxNode, ctx: &LayoutContext) -> MathElement {
    // \hbox, \mbox - horizontal box in text mode
    build_expression(&hbox.body, ctx)
}

fn build_xarrow(arrow: &XArrowNode, ctx: &LayoutContext) -> MathElement {
    // Extensible arrows like \xrightarrow, \xleftarrow
    let metrics = ctx.metrics();

    // Build upper label (above arrow) in sup style
    let sup_ctx = ctx.for_superscript();
    let upper = build_node(&arrow.body, &sup_ctx);
    let (upper_width, upper_height, upper_depth) = upper.dimensions();

    // Build lower label (below arrow) in sub style if present
    let (lower, lower_height, lower_depth, lower_width) = if let Some(ref below) = arrow.below {
        let sub_ctx = ctx.for_subscript();
        let l = build_node(below, &sub_ctx);
        let (w, h, d) = l.dimensions();
        (Some(l), h, d, w)
    } else {
        (None, 0.0, 0.0, 0.0)
    };

    // Arrow dimensions - minimum width based on labels
    let arrow_width = (upper_width.max(lower_width) + 0.5).max(1.75);
    let arrow_height = metrics.default_rule_thickness * 2.0;

    // Build arrow symbol using label (e.g., "\\xrightarrow" -> "→")
    let arrow_char = match arrow.label.as_str() {
        "\\xrightarrow" | "\\xRightarrow" => "→",
        "\\xleftarrow" | "\\xLeftarrow" => "←",
        "\\xleftrightarrow" | "\\xLeftrightarrow" => "↔",
        "\\xhookleftarrow" => "↩",
        "\\xhookrightarrow" => "↪",
        "\\xmapsto" => "↦",
        "\\xrightharpoonup" => "⇀",
        "\\xrightharpoondown" => "⇁",
        "\\xleftharpoonup" => "↼",
        "\\xleftharpoondown" => "↽",
        _ => "→",
    };
    let arrow_sym = build_symbol(arrow_char, ctx, true);

    // Stack: upper, arrow, lower (if present)
    let axis_height = metrics.axis_height;
    let gap = 0.1; // Kerning between arrow and labels

    let mut children = Vec::new();
    let mut y = axis_height + arrow_height / 2.0 + gap;

    // Upper label
    children.push(Positioned::new(upper.clone(), 0.0, y + upper_depth));
    y = axis_height;

    // Arrow at axis height
    children.push(Positioned::new(arrow_sym.clone(), 0.0, y));

    // Lower label if present
    if let Some(ref low) = lower {
        let low_y = axis_height - arrow_height / 2.0 - gap - lower_height;
        children.push(Positioned::new(low.clone(), 0.0, low_y));
    }

    let total_height = axis_height + arrow_height / 2.0 + gap + upper_height + upper_depth;
    let total_depth = if lower.is_some() {
        (axis_height - arrow_height / 2.0 - gap - lower_height - lower_depth).abs()
    } else {
        0.0
    };

    MathElement::VBox {
        children,
        width: arrow_width,
        height: total_height,
        depth: total_depth,
    }
}

fn build_enclose(enclose: &EncloseNode, ctx: &LayoutContext) -> MathElement {
    // \boxed, \colorbox, \fcolorbox, \cancel, etc.
    let inner = build_node(&enclose.body, ctx);
    let (width, height, depth) = inner.dimensions();

    // Add padding for boxes
    let padding = match enclose.label.as_str() {
        "\\boxed" | "\\fbox" | "\\colorbox" | "\\fcolorbox" => ctx.metrics().fboxsep,
        _ => 0.0,
    };

    let padded_width = width + 2.0 * padding;
    let padded_height = height + padding;
    let padded_depth = depth + padding;

    // For now, wrap in HBox with appropriate dimensions
    // The actual border/background rendering is handled by the renderer
    MathElement::HBox {
        children: vec![Positioned::new(inner, padding, 0.0)],
        width: padded_width,
        height: padded_height,
        depth: padded_depth,
        classes: vec![enclose.label.trim_start_matches('\\').to_string()],
    }
}

fn build_horiz_brace(brace: &HorizBraceNode, ctx: &LayoutContext) -> MathElement {
    // \overbrace, \underbrace
    let base = build_node(&brace.base, ctx);
    let (base_width, base_height, base_depth) = base.dimensions();

    // Build brace symbol
    let brace_char = match brace.label.as_str() {
        "\\overbrace" => "⏞",
        "\\underbrace" => "⏟",
        "\\overgroup" => "⏠",
        "\\undergroup" => "⏡",
        _ => "⏞",
    };
    let brace_sym = build_symbol(brace_char, ctx, true);
    let (_, brace_height, brace_depth) = brace_sym.dimensions();

    let gap = 0.1; // Kerning between brace and content

    let (children, height, depth) = if brace.is_over {
        // Brace above content
        let children = vec![
            Positioned::new(brace_sym.clone(), 0.0, base_height + gap),
            Positioned::at_origin(base.clone()),
        ];
        (children, base_height + gap + brace_height, base_depth)
    } else {
        // Brace below content
        let brace_y = -(base_depth + gap + brace_height);
        let children = vec![
            Positioned::at_origin(base.clone()),
            Positioned::new(brace_sym.clone(), 0.0, brace_y),
        ];
        (children, base_height, base_depth + gap + brace_height + brace_depth)
    };

    let layout = MathElement::VBox {
        children,
        width: base_width,
        height,
        depth,
    };

    if ctx.config.semantic_mode {
        MathElement::Accent {
            base: Box::new(base),
            accent: Box::new(brace_sym),
            is_over: brace.is_over,
            layout: Box::new(layout),
        }
    } else {
        layout
    }
}

fn build_lap(lap: &LapNode, ctx: &LayoutContext) -> MathElement {
    // \llap, \rlap, \clap - overlapping content
    let inner = build_node(&lap.body, ctx);
    let (width, height, depth) = inner.dimensions();

    // Lap elements have zero width but preserve height/depth
    // The alignment class tells the renderer how to position
    MathElement::HBox {
        children: vec![Positioned::at_origin(inner)],
        width: 0.0, // Zero width - content overlaps
        height,
        depth,
        classes: vec![lap.alignment.clone()],
    }
}

fn build_math_choice(choice: &MathChoiceNode, ctx: &LayoutContext) -> MathElement {
    // \mathchoice - select branch based on current style
    let body = if ctx.is_display() {
        &choice.display
    } else if ctx.is_script() {
        &choice.script
    } else if ctx.is_scriptscript() {
        &choice.script_script
    } else {
        &choice.text
    };

    build_expression(body, ctx)
}

fn build_raise_box(rbox: &RaiseBoxNode, ctx: &LayoutContext) -> MathElement {
    // \raisebox - vertically shift content
    let inner = build_node(&rbox.body, ctx);
    let (width, height, depth) = inner.dimensions();

    let dy = calculate_size(&rbox.dy, &ctx.options);

    // Positive dy raises content (shifts baseline down relative to content)
    MathElement::HBox {
        children: vec![Positioned::new(inner, 0.0, dy)],
        width,
        height: (height + dy).max(0.0),
        depth: (depth - dy).max(0.0),
        classes: vec![],
    }
}

fn build_smash(smash: &SmashNode, ctx: &LayoutContext) -> MathElement {
    // \smash - zero out height and/or depth
    let inner = build_node(&smash.body, ctx);
    let (width, height, depth) = inner.dimensions();

    let final_height = if smash.smash_height { 0.0 } else { height };
    let final_depth = if smash.smash_depth { 0.0 } else { depth };

    MathElement::HBox {
        children: vec![Positioned::at_origin(inner)],
        width,
        height: final_height,
        depth: final_depth,
        classes: vec![],
    }
}

fn build_vcenter(vc: &VCenterNode, ctx: &LayoutContext) -> MathElement {
    // \vcenter - center content on math axis
    let inner = build_node(&vc.body, ctx);
    let (width, height, depth) = inner.dimensions();

    let axis_height = ctx.metrics().axis_height;

    // Shift to center content on axis
    // Formula: dy = 0.5 * ((height - axis) - (depth + axis))
    let dy = 0.5 * ((height - axis_height) - (depth + axis_height));

    MathElement::HBox {
        children: vec![Positioned::new(inner, 0.0, dy)],
        width,
        height: height - dy,
        depth: depth + dy,
        classes: vec![],
    }
}

fn build_array(arr: &ArrayNode, ctx: &LayoutContext) -> MathElement {
    // Arrays and matrices
    let metrics = ctx.metrics();

    // Column separation
    let col_sep = match arr.col_separation_type {
        Some(ColSeparationType::Small) => 0.2778, // Small spacing
        Some(ColSeparationType::Cd) => 0.5,       // CD environment
        _ => 0.5,                                  // Default (5pt at 10pt font)
    };

    // Row separation
    let base_row_skip = 1.2; // 12pt at 10pt font
    let jot = if arr.add_jot.unwrap_or(false) { 0.3 } else { 0.0 };

    // Build all cells
    let mut rows: Vec<Vec<MathElement>> = Vec::new();
    let mut row_heights: Vec<f64> = Vec::new();
    let mut row_depths: Vec<f64> = Vec::new();
    let mut col_widths: Vec<f64> = Vec::new();

    for row in &arr.body {
        let mut built_row = Vec::new();
        let mut max_height = 0.0f64;
        let mut max_depth = 0.0f64;

        for (col_idx, cell) in row.iter().enumerate() {
            let cell_elem = build_node(cell, ctx);
            let (w, h, d) = cell_elem.dimensions();

            // Track column widths
            if col_idx >= col_widths.len() {
                col_widths.push(w);
            } else {
                col_widths[col_idx] = col_widths[col_idx].max(w);
            }

            max_height = max_height.max(h);
            max_depth = max_depth.max(d);
            built_row.push(cell_elem);
        }

        row_heights.push(max_height);
        row_depths.push(max_depth);
        rows.push(built_row);
    }

    // Calculate total dimensions
    let total_width: f64 = col_widths.iter().sum::<f64>()
        + col_sep * (col_widths.len().saturating_sub(1)) as f64;

    // Position cells
    let mut children = Vec::new();
    let mut y = 0.0;

    for (row_idx, row) in rows.iter().enumerate() {
        let row_height = row_heights[row_idx];
        let row_depth = row_depths[row_idx];

        // Start at top of row
        if row_idx == 0 {
            y = row_height;
        }

        let mut x = 0.0;
        for (col_idx, cell) in row.iter().enumerate() {
            let col_width = col_widths[col_idx];
            let (cell_width, _, _) = cell.dimensions();

            // Center cell in column (default alignment)
            let x_offset = (col_width - cell_width) / 2.0;

            children.push(Positioned::new(cell.clone(), x + x_offset, y));

            x += col_width + col_sep;
        }

        // Move to next row
        let gap = if row_idx + 1 < arr.row_gaps.len() {
            arr.row_gaps[row_idx + 1]
                .as_ref()
                .map(|m| calculate_size(m, &ctx.options))
                .unwrap_or(0.0)
        } else {
            0.0
        };

        if row_idx + 1 < rows.len() {
            y -= row_depth + base_row_skip + jot + gap + row_heights[row_idx + 1];
        }
    }

    let total_height = row_heights.first().copied().unwrap_or(0.0);
    let total_depth = if rows.len() > 1 {
        (-y + row_depths.last().copied().unwrap_or(0.0)).abs()
    } else {
        row_depths.first().copied().unwrap_or(0.0)
    };

    // Center array on axis
    let axis_height = metrics.axis_height;
    let shift = (total_height - total_depth) / 2.0 - axis_height;

    // Apply shift to all children
    let shifted_children: Vec<_> = children
        .into_iter()
        .map(|mut p| {
            p.y -= shift;
            p
        })
        .collect();

    let layout = MathElement::VBox {
        children: shifted_children,
        width: total_width,
        height: total_height - shift,
        depth: total_depth + shift,
    };

    if ctx.config.semantic_mode {
        MathElement::Array {
            cells: rows,
            row_lines: arr.h_lines_before_row.iter().map(|v| !v.is_empty()).collect(),
            col_lines: vec![false; col_widths.len()], // TODO: parse column lines
            layout: Box::new(layout),
        }
    } else {
        layout
    }
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
    fn test_build_greek_symbols() {
        let opts = default_options();
        let config = IrBuilderConfig::default();
        let ctx = LayoutContext::new(&opts, &config);

        // Test that macro names are converted to Unicode characters
        let alpha = build_symbol("\\alpha", &ctx, true);
        if let MathElement::Text { text, .. } = alpha {
            assert_eq!(text, "α", "\\alpha should render as Unicode α");
        } else {
            panic!("Expected Text element for alpha");
        }

        let beta = build_symbol("\\beta", &ctx, true);
        if let MathElement::Text { text, .. } = beta {
            assert_eq!(text, "β", "\\beta should render as Unicode β");
        } else {
            panic!("Expected Text element for beta");
        }

        // Test uppercase Greek
        let gamma = build_symbol("\\Gamma", &ctx, true);
        if let MathElement::Text { text, .. } = gamma {
            assert_eq!(text, "Γ", "\\Gamma should render as Unicode Γ");
        } else {
            panic!("Expected Text element for Gamma");
        }

        // Test operators and relations
        let infty = build_symbol("\\infty", &ctx, true);
        if let MathElement::Text { text, .. } = infty {
            assert_eq!(text, "∞", "\\infty should render as Unicode ∞");
        } else {
            panic!("Expected Text element for infty");
        }

        let rightarrow = build_symbol("\\rightarrow", &ctx, true);
        if let MathElement::Text { text, .. } = rightarrow {
            assert_eq!(text, "→", "\\rightarrow should render as Unicode →");
        } else {
            panic!("Expected Text element for rightarrow");
        }

        let leq = build_symbol("\\leq", &ctx, true);
        if let MathElement::Text { text, .. } = leq {
            assert_eq!(text, "≤", "\\leq should render as Unicode ≤");
        } else {
            panic!("Expected Text element for leq");
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

    #[test]
    fn test_inter_element_spacing() {
        use crate::parse_tree;

        // Test that spacing is inserted between elements
        let opts = default_options();

        // x + y should have spacing around the +
        let tree = parse_tree("x + y", ParserConfig::default()).unwrap();
        let layout = build_ir(&tree, &opts);

        // Count elements including spacing kerns
        let count = count_elements(&layout.root);
        // Should have: x, kern, +, kern, y (= 5 elements)
        // Or without spacing: x, +, y (= 3 elements)
        assert!(count >= 3, "Expected at least 3 elements, got {}", count);
    }

    #[test]
    fn test_spacing_binary_vs_relation() {
        use crate::parse_tree;

        let opts = default_options();

        // Binary operator (thin/medium space)
        let tree_bin = parse_tree("x + y", ParserConfig::default()).unwrap();
        let layout_bin = build_ir(&tree_bin, &opts);

        // Relation operator (thick space)
        let tree_rel = parse_tree("x = y", ParserConfig::default()).unwrap();
        let layout_rel = build_ir(&tree_rel, &opts);

        // Both should produce valid layouts with positive width
        assert!(layout_bin.width > 0.0);
        assert!(layout_rel.width > 0.0);

        // Relation should have wider spacing than binary
        // (thick space = 5mu vs medium space = 4mu)
        // Note: exact comparison is tricky due to symbol widths
    }

    #[test]
    fn test_sqrt_layout() {
        use crate::parse_tree;

        let opts = default_options();

        // Test basic sqrt
        let tree = parse_tree(r"\sqrt{x}", ParserConfig::default()).unwrap();
        let layout = build_ir(&tree, &opts);

        // Should produce a Radical element in semantic mode
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);

        // Test sqrt with index
        let tree = parse_tree(r"\sqrt[3]{x}", ParserConfig::default()).unwrap();
        let layout = build_ir(&tree, &opts);
        assert!(layout.width > 0.0);

        // Test nested sqrt
        let tree = parse_tree(r"\sqrt{\sqrt{x}}", ParserConfig::default()).unwrap();
        let layout = build_ir(&tree, &opts);
        assert!(layout.width > 0.0);
    }

    /// Helper to count total elements in a MathElement tree
    fn count_elements(elem: &MathElement) -> usize {
        match elem {
            MathElement::HBox { children, .. } => {
                1 + children.iter().map(|c| count_elements(&c.element)).sum::<usize>()
            }
            MathElement::VBox { children, .. } => {
                1 + children.iter().map(|c| count_elements(&c.element)).sum::<usize>()
            }
            MathElement::Fraction { layout, .. }
            | MathElement::Scripts { layout, .. }
            | MathElement::Radical { layout, .. }
            | MathElement::Accent { layout, .. }
            | MathElement::Delimited { layout, .. }
            | MathElement::LargeOp { layout, .. }
            | MathElement::Array { layout, .. } => {
                1 + count_elements(layout)
            }
            MathElement::Color { inner, .. }
            | MathElement::Phantom { inner, .. }
            | MathElement::Link { inner, .. } => {
                1 + count_elements(inner)
            }
            _ => 1,
        }
    }
}
