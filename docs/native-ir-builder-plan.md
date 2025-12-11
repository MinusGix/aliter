# Native IR Builder Implementation Plan

## Overview

This document outlines a plan to implement a native IR builder that generates `MathLayout` directly from `ParseNode` trees, bypassing the HTML intermediate representation. This addresses the feedback about:

1. Confusing VBox y-values (CSS top offsets)
2. Zero widths (HTML relies on auto-sizing)
3. Missing sup/sub y-offsets
4. Lost semantic information

## Architecture

### Current Flow (to be preserved for HTML output)
```
ParseNode → HTML Builder → HtmlNode → (optional) IR Conversion
```

### New Flow (native IR)
```
ParseNode → IR Builder → MathLayout
```

The native IR builder will:
- Compute real widths from `CharacterMetrics`
- Calculate explicit y-offsets for all positioned elements
- Optionally preserve semantic structure in `MathElement` variants
- Use the same font metrics and positioning algorithms as the HTML builder

## Implementation Phases

### Phase 1: Core Infrastructure

#### 1.1 New Module Structure

Create `src/ir/mod.rs` (refactored from existing `src/ir.rs`):

```
src/ir/
├── mod.rs           # Re-exports and MathElement/MathLayout types
├── types.rs         # Core types (MathElement, Positioned, TextStyle, etc.)
├── from_html.rs     # Existing HTML→IR conversion (moved)
├── to_html.rs       # Existing IR→HTML rendering (moved)
├── builder.rs       # NEW: Native IR builder
└── layout.rs        # NEW: Layout computation helpers
```

#### 1.2 Enhanced MathElement (Optional Semantic Variants)

Extend `MathElement` with optional semantic variants that wrap layout:

```rust
pub enum MathElement {
    // === Existing layout variants (unchanged) ===
    Text { text: String, style: TextStyle },
    HBox { children: Vec<Positioned<MathElement>>, width: f64, height: f64, depth: f64, classes: Vec<String> },
    VBox { children: Vec<Positioned<MathElement>>, width: f64, height: f64, depth: f64 },
    Rule { width: f64, height: f64, shift: f64, style: LineStyle, color: Option<Color> },
    Path { path_data: Cow<'static, str>, width: f64, height: f64, shift: f64 },
    Kern { width: f64 },
    Phantom { inner: Box<MathElement> },
    Color { color: Color, inner: Box<MathElement> },
    Link { href: String, inner: Box<MathElement> },
    Image { src: String, alt: String, width: f64, height: f64 },
    Breakable { children: Vec<Positioned<MathElement>>, width: f64, height: f64, depth: f64 },

    // === NEW: Semantic variants ===
    // These contain both semantic info AND computed layout

    /// Fraction with numerator, denominator, and optional bar
    Fraction {
        numerator: Box<Positioned<MathElement>>,
        denominator: Box<Positioned<MathElement>>,
        bar: Option<FractionBar>,
        /// Pre-computed layout as VBox (for renderers that don't need semantics)
        layout: Box<MathElement>,
    },

    /// Superscript/subscript
    Scripts {
        base: Option<Box<Positioned<MathElement>>>,
        superscript: Option<Box<Positioned<MathElement>>>,
        subscript: Option<Box<Positioned<MathElement>>>,
        /// Pre-computed layout
        layout: Box<MathElement>,
    },

    /// Square root
    Radical {
        radicand: Box<MathElement>,
        index: Option<Box<MathElement>>,
        /// Pre-computed layout including surd and vinculum
        layout: Box<MathElement>,
    },

    /// Accent (hat, tilde, etc.)
    Accent {
        base: Box<MathElement>,
        accent: Box<MathElement>,
        is_over: bool,
        layout: Box<MathElement>,
    },

    /// Delimited expression \left...\right
    Delimited {
        left: Option<Box<MathElement>>,
        right: Option<Box<MathElement>>,
        body: Box<MathElement>,
        layout: Box<MathElement>,
    },

    /// Large operator with limits
    LargeOp {
        nucleus: Box<MathElement>,
        superscript: Option<Box<Positioned<MathElement>>>,
        subscript: Option<Box<Positioned<MathElement>>>,
        limits: bool,
        layout: Box<MathElement>,
    },
}

pub struct FractionBar {
    pub thickness: f64,
    pub color: Option<Color>,
    pub style: LineStyle,
}
```

The semantic variants include a `layout` field containing the pre-computed layout tree. Renderers can:
- Use the semantic fields for native rendering
- Fall back to `layout` for generic rendering
- Mix both approaches

#### 1.3 Builder Configuration

```rust
pub struct IrBuilderConfig {
    /// Whether to emit semantic variants (Fraction, Scripts, etc.)
    /// or pure layout variants (HBox, VBox, etc.)
    pub semantic_mode: bool,

    /// Whether to include CSS classes in HBox (for debugging/compatibility)
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
```

### Phase 2: Layout Computation Module

#### 2.1 Layout Helpers (`src/ir/layout.rs`)

Port/adapt the key layout computations from `build_common.rs`:

```rust
/// Context for IR layout computation
pub struct LayoutContext<'a> {
    pub options: &'a Options,
    pub config: &'a IrBuilderConfig,
}

impl LayoutContext<'_> {
    /// Get font metrics for current style
    pub fn metrics(&self) -> &FontMetrics {
        self.options.font_metrics()
    }

    /// Get character metrics for a symbol
    pub fn char_metrics(&self, ch: char, font: &str) -> Option<CharacterMetrics> {
        get_character_metrics(ch, font, self.options.style.mode())
    }
}

/// Compute dimensions for a text element
pub fn text_dimensions(text: &str, style: &TextStyle, ctx: &LayoutContext) -> (f64, f64, f64) {
    let font_name = style.font.as_ref()
        .map(|f| f.as_str())
        .unwrap_or("Main-Regular");

    let mut width = 0.0;
    let mut height = 0.0;
    let mut depth = 0.0;

    for ch in text.chars() {
        if let Some(metrics) = ctx.char_metrics(ch, font_name) {
            width += metrics.width * style.size;
            height = height.max(metrics.height * style.size);
            depth = depth.max(metrics.depth * style.size);
        } else {
            // Fallback for unknown characters
            width += 0.5 * style.size;
            height = height.max(0.7 * style.size);
            depth = depth.max(0.2 * style.size);
        }
    }

    // Add italic correction if applicable
    width += style.italic_correction;

    (width, height, depth)
}

/// Stack elements vertically with explicit positioning
/// Returns (VBox element, positions of each child)
pub fn stack_vertical(
    children: &[MathElement],
    params: VStackParams,
    ctx: &LayoutContext,
) -> MathElement {
    // ... implementation adapted from make_v_list
}

pub enum VStackParams {
    /// Each child specifies its baseline shift from the composite baseline
    IndividualShift(Vec<f64>),
    /// Position from top
    Top { amount: f64 },
    /// Position from bottom
    Bottom { amount: f64 },
    /// Aligned with first child's baseline
    FirstBaseline,
}

/// Compute superscript/subscript positions
pub fn compute_script_positions(
    base_height: f64,
    base_depth: f64,
    sup: Option<(f64, f64)>,  // (height, depth)
    sub: Option<(f64, f64)>,  // (height, depth)
    ctx: &LayoutContext,
) -> ScriptPositions {
    let metrics = ctx.metrics();

    // Use TeX positioning rules (TeXbook Appendix G, Rule 18)
    let mut sup_shift = metrics.sup1;  // or sup2/sup3 depending on style
    let mut sub_shift = metrics.sub1;  // or sub2 depending on style

    // Apply clearance rules...
    // (adapted from existing supsub and op.rs logic)

    ScriptPositions { sup_shift, sub_shift }
}

pub struct ScriptPositions {
    /// Distance from baseline to superscript baseline (positive = up)
    pub sup_shift: f64,
    /// Distance from baseline to subscript baseline (positive = down)
    pub sub_shift: f64,
}
```

### Phase 3: Node Builders

#### 3.1 Main Builder Entry Point (`src/ir/builder.rs`)

```rust
use crate::parse_node::ParseNode;
use crate::Options;
use super::{MathElement, MathLayout, Positioned};
use super::layout::LayoutContext;

/// Build IR directly from a parse tree
pub fn build_ir(tree: &[ParseNode], options: &Options) -> MathLayout {
    build_ir_with_config(tree, options, &IrBuilderConfig::default())
}

pub fn build_ir_with_config(
    tree: &[ParseNode],
    options: &Options,
    config: &IrBuilderConfig,
) -> MathLayout {
    let ctx = LayoutContext { options, config };
    let root = build_expression(tree, &ctx);
    let (width, height, depth) = root.dimensions();

    MathLayout {
        root,
        display_mode: options.style.is_display(),
        width,
        height,
        depth,
    }
}

fn build_expression(nodes: &[ParseNode], ctx: &LayoutContext) -> MathElement {
    let mut children = Vec::new();
    let mut x_offset = 0.0;

    for node in nodes {
        let element = build_node(node, ctx);
        let width = element.width();
        children.push(Positioned::new(element, x_offset, 0.0));
        x_offset += width;

        // Add inter-element spacing based on atom types
        // (port spacing logic from html.rs)
    }

    let (_, height, depth) = compute_hbox_dimensions(&children);

    MathElement::HBox {
        children,
        width: x_offset,
        height,
        depth,
        classes: vec![],
    }
}

fn build_node(node: &ParseNode, ctx: &LayoutContext) -> MathElement {
    match node {
        ParseNode::MathOrd(ord) => build_math_ord(ord, ctx),
        ParseNode::TextOrd(ord) => build_text_ord(ord, ctx),
        ParseNode::Atom(atom) => build_atom(atom, ctx),
        ParseNode::SupSub(supsub) => build_supsub(supsub, ctx),
        ParseNode::GenFrac(frac) => build_fraction(frac, ctx),
        ParseNode::Sqrt(sqrt) => build_sqrt(sqrt, ctx),
        ParseNode::Accent(accent) => build_accent(accent, ctx),
        ParseNode::Op(op) => build_op(op, ctx),
        ParseNode::OrdGroup(group) => build_expression(&group.body, ctx),
        ParseNode::Spacing(sp) => build_spacing(sp, ctx),
        ParseNode::Kern(kern) => build_kern(kern, ctx),
        // ... other node types
        _ => {
            // Fallback: convert to HTML then to IR
            // This allows incremental implementation
            todo!("Node type {:?} not yet implemented", node.typ())
        }
    }
}
```

#### 3.2 Symbol/Text Builders

```rust
fn build_math_ord(ord: &MathOrdNode, ctx: &LayoutContext) -> MathElement {
    let font = determine_font(ord, ctx);
    let ch = ord.text.chars().next().unwrap();

    if let Some(metrics) = ctx.char_metrics(ch, font) {
        MathElement::Text {
            text: ord.text.clone(),
            style: TextStyle {
                font: Some(Font::from_name(font)),
                size: ctx.options.size_multiplier(),
                color: ctx.options.get_color(),
                italic_correction: metrics.italic,
                skew: metrics.skew,
            },
        }
    } else {
        // Fallback
        MathElement::Text {
            text: ord.text.clone(),
            style: TextStyle {
                font: Some(Font::MathItalic),
                size: ctx.options.size_multiplier(),
                ..Default::default()
            },
        }
    }
}
```

#### 3.3 Fraction Builder

```rust
fn build_fraction(frac: &GenFracNode, ctx: &LayoutContext) -> MathElement {
    let metrics = ctx.metrics();

    // Build numerator and denominator in appropriate styles
    let numer_ctx = ctx.with_style(ctx.options.style.frac_num());
    let denom_ctx = ctx.with_style(ctx.options.style.frac_den());

    let numer = build_node(&frac.numer, &numer_ctx);
    let denom = build_node(&frac.denom, &denom_ctx);

    // Compute positioning (TeX Rule 15)
    let is_display = ctx.options.style.is_display();
    let (num_shift, denom_shift, rule_width) = compute_frac_positions(
        &numer, &denom, frac.has_bar_line, is_display, metrics
    );

    // Build the rule if needed
    let bar = if frac.has_bar_line {
        let thickness = frac.bar_size
            .as_ref()
            .map(|m| calculate_size(m, ctx.options))
            .unwrap_or(metrics.default_rule_thickness);
        Some(FractionBar {
            thickness,
            color: None,
            style: LineStyle::Solid,
        })
    } else {
        None
    };

    // Compute layout as VBox
    let axis_height = metrics.axis_height;
    let numer_pos = Positioned::new(numer.clone(), 0.0, num_shift);
    let denom_pos = Positioned::new(denom.clone(), 0.0, -denom_shift);

    let mut children = vec![numer_pos.clone(), denom_pos.clone()];
    if let Some(ref bar) = bar {
        let rule = MathElement::Rule {
            width: numer.width().max(denom.width()),
            height: bar.thickness,
            shift: axis_height,
            style: bar.style,
            color: bar.color.clone(),
        };
        // Insert rule between numer and denom
        children.insert(1, Positioned::new(rule, 0.0, axis_height));
    }

    let width = numer.width().max(denom.width());
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
```

#### 3.4 Superscript/Subscript Builder

```rust
fn build_supsub(supsub: &SupSubNode, ctx: &LayoutContext) -> MathElement {
    let metrics = ctx.metrics();

    // Build base
    let base = supsub.base.as_ref().map(|b| build_node(b, ctx));
    let base_height = base.as_ref().map(|b| b.height()).unwrap_or(0.0);
    let base_depth = base.as_ref().map(|b| b.depth()).unwrap_or(0.0);

    // Build scripts in smaller style
    let script_ctx = ctx.with_style(ctx.options.style.sup());
    let sup = supsub.sup.as_ref().map(|s| build_node(s, &script_ctx));
    let sub = supsub.sub.as_ref().map(|s| build_node(s, &script_ctx));

    // Compute positions using TeX rules (Rule 18)
    let positions = compute_script_positions(
        base_height, base_depth,
        sup.as_ref().map(|s| (s.height(), s.depth())),
        sub.as_ref().map(|s| (s.height(), s.depth())),
        ctx,
    );

    // Build positioned elements
    let base_pos = base.map(|b| {
        let w = b.width();
        Positioned::new(b, 0.0, 0.0)
    });

    let base_width = base_pos.as_ref().map(|b| b.element.width()).unwrap_or(0.0);

    let sup_pos = sup.map(|s| {
        Positioned::new(s, base_width, positions.sup_shift)
    });

    let sub_pos = sub.map(|s| {
        Positioned::new(s, base_width, -positions.sub_shift)
    });

    // Compute overall dimensions
    let width = base_width
        + sup_pos.as_ref().map(|s| s.element.width()).unwrap_or(0.0)
            .max(sub_pos.as_ref().map(|s| s.element.width()).unwrap_or(0.0));

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
        width,
        height,
        depth,
        classes: vec![],
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
```

### Phase 4: Additional Node Types

Implement builders for remaining node types in priority order:

1. **High Priority** (common in math)
   - `Sqrt` - Square roots with surd and vinculum
   - `Accent` - Accents (hat, tilde, bar, etc.)
   - `Op` - Large operators with limits
   - `LeftRight` - Delimited expressions
   - `Array` - Matrices and alignments

2. **Medium Priority**
   - `Overline`, `Underline`
   - `HorizBrace`
   - `Enclose` (box, cancel, etc.)
   - `XArrow` - Extensible arrows
   - `Spacing`, `Kern`

3. **Lower Priority**
   - `Phantom`, `HPhantom`, `VPhantom`
   - `Smash`
   - `RaiseBox`
   - `VCenter`
   - `Href`, `Html`
   - `IncludeGraphics`
   - `Color`, `Font`, `Sizing`, `Styling`

### Phase 5: Testing & Validation

#### 5.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_width_computation() {
        let layout = build_ir(&parse("x").unwrap(), &default_options());
        // Width should be actual character width from font metrics
        assert!((layout.width - 0.572).abs() < 0.01); // 'x' width in Math-Italic
    }

    #[test]
    fn test_supsub_positioning() {
        let layout = build_ir(&parse("x^2").unwrap(), &default_options());

        // Find the superscript element and verify y-offset
        if let MathElement::Scripts { superscript, .. } = &layout.root {
            let sup = superscript.as_ref().unwrap();
            assert!(sup.y > 0.0); // Superscript should be above baseline
            assert!((sup.y - 0.413).abs() < 0.05); // Should match sup1 metric
        }
    }

    #[test]
    fn test_fraction_layout() {
        let layout = build_ir(&parse(r"\frac{1}{2}").unwrap(), &default_options());

        if let MathElement::Fraction { numerator, denominator, bar, .. } = &layout.root {
            assert!(numerator.y > 0.0);  // Numerator above baseline
            assert!(denominator.y < 0.0); // Denominator below baseline
            assert!(bar.is_some());
        }
    }
}
```

#### 5.2 Comparison Tests

Compare native IR output against HTML→IR conversion to ensure equivalent layout:

```rust
#[test]
fn test_native_vs_html_conversion() {
    let expressions = ["x", "x+y", r"\frac{1}{2}", r"x^2_i"];

    for expr in expressions {
        let tree = parse(expr).unwrap();
        let opts = default_options();

        // Native IR
        let native = build_ir(&tree, &opts);

        // HTML → IR
        let html = render_to_html_tree(expr, &opts);
        let converted = from_html::convert(&html);

        // Compare dimensions (should be close)
        assert!((native.width - converted.width()).abs() < 0.1);
        assert!((native.height - converted.height()).abs() < 0.1);
    }
}
```

### Phase 6: API & Integration

#### 6.1 Public API

Add to `src/lib.rs`:

```rust
/// Build IR layout directly from LaTeX input
pub fn render_to_ir(input: &str, config: ParserConfig) -> Result<MathLayout, ParseError> {
    let tree = parse_tree(input, config.clone())?;
    let options = Options::from_parser_conf(&config);
    Ok(ir::builder::build_ir(&tree, &options))
}

/// Build IR layout with custom builder configuration
pub fn render_to_ir_with_config(
    input: &str,
    parser_config: ParserConfig,
    ir_config: ir::IrBuilderConfig,
) -> Result<MathLayout, ParseError> {
    let tree = parse_tree(input, parser_config.clone())?;
    let options = Options::from_parser_conf(&parser_config);
    Ok(ir::builder::build_ir_with_config(&tree, &options, &ir_config))
}
```

#### 6.2 Feature Flag

```toml
# Cargo.toml
[features]
default = ["html", "mathml"]
html = []
mathml = []
ir = []  # Native IR builder
```

## Implementation Order

1. **Week 1**: Core infrastructure
   - Refactor `ir.rs` into module structure
   - Add semantic variants to `MathElement`
   - Implement `LayoutContext` and basic layout helpers

2. **Week 2**: Basic node builders
   - `MathOrd`, `TextOrd`, `Atom` (text elements with proper widths)
   - `Spacing`, `Kern`
   - `OrdGroup` (expression builder)

3. **Week 3**: Script positioning
   - `SupSub` with explicit y-offsets
   - Port positioning algorithms from HTML builder

4. **Week 4**: Fractions and vertically stacked content
   - `GenFrac`
   - `VBox` layout helpers

5. **Week 5**: Complex structures
   - `Sqrt`
   - `Accent`
   - `Op` with limits

6. **Week 6**: Remaining nodes and testing
   - Lower priority nodes
   - Comprehensive testing
   - Documentation

## Success Criteria

1. **Widths**: All text elements have accurate widths from font metrics
2. **Positioning**: All y-offsets are explicit and correct (no CSS class detection needed)
3. **Semantics**: Semantic variants available for common structures
4. **Compatibility**: Layout matches HTML output dimensions within tolerance
5. **Performance**: Native IR builds faster than HTML→IR conversion

## Open Questions

1. **Semantic variant granularity**: Should we have variants for all structures or just common ones?
2. **CSS class preservation**: Should HBox preserve classes for debugging, or omit entirely?
3. **Incremental migration**: Keep HTML→IR path for fallback during implementation?
4. **Array/matrix handling**: These are complex - defer to later phase?

## References

- `src/build_common.rs` - VList construction, symbol lookup
- `src/html.rs` - Expression building, spacing computation
- `src/functions/genfrac.rs` - Fraction positioning (Rule 15)
- `src/functions/op.rs` - Operator with limits positioning
- `src/font_metrics.rs` - Character and global font metrics
- TeXbook Appendix G - TeX positioning rules
