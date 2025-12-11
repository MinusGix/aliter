//! IR (Intermediate Representation) Demo
//!
//! This example demonstrates the native IR builder and explains the output format.
//!
//! Run with: cargo run --example ir_demo

use aliter::{ir, parse_tree, parser::ParserConfig, Options};

fn main() {
    println!("=== Aliter IR Demo ===\n");

    // Basic setup
    let config = ParserConfig::default();
    let options = Options::from_parser_conf(&config);

    // Demo 1: Simple expression
    demo_simple(&options);

    // Demo 2: Fraction with semantic info
    demo_fraction(&options);

    // Demo 3: Superscript/subscript
    demo_scripts(&options);

    // Demo 4: Square root
    demo_sqrt(&options);

    // Demo 5: Inter-element spacing
    demo_spacing(&options);

    // Demo 6: Walking the tree
    demo_walker(&options);

    // Demo 7: Layout-only mode (no semantic info)
    demo_layout_only(&options);
}

fn demo_simple(options: &Options) {
    println!("--- Demo 1: Simple Expression ---");
    println!("Input: x + y\n");

    let tree = parse_tree("x + y", ParserConfig::default()).unwrap();
    let layout = ir::build_ir(&tree, options);

    println!("MathLayout {{");
    println!("  display_mode: {},", layout.display_mode);
    println!("  width: {:.4} em,", layout.width);
    println!("  height: {:.4} em (above baseline),", layout.height);
    println!("  depth: {:.4} em (below baseline),", layout.depth);
    println!("}}");
    println!();

    // Explain coordinate system
    println!("Coordinate System:");
    println!("  - Origin (0,0) is at the LEFT edge of the expression, ON the baseline");
    println!("  - X increases to the right");
    println!("  - Y increases upward (positive = above baseline)");
    println!("  - All measurements are in em units");
    println!();
}

fn demo_fraction(options: &Options) {
    println!("--- Demo 2: Fraction (Semantic Mode) ---");
    println!("Input: \\frac{{a}}{{b}}\n");

    let tree = parse_tree(r"\frac{a}{b}", ParserConfig::default()).unwrap();
    let layout = ir::build_ir(&tree, options);

    // Find the Fraction element (it's inside an HBox)
    let fraction = layout.walk().find(|item| {
        matches!(item.element, ir::MathElement::Fraction { .. })
    });

    if let Some(frac_item) = fraction {
        if let ir::MathElement::Fraction { numerator, denominator, bar, .. } = frac_item.element {
            println!("Fraction Structure:");
            println!("  numerator:");
            println!("    x: {:.4} em (horizontal offset)", numerator.x);
            println!("    y: {:.4} em (vertical offset from baseline)", numerator.y);
            println!("    content: {:?}", element_type(&numerator.element));
            println!();
            println!("  denominator:");
            println!("    x: {:.4} em", denominator.x);
            println!("    y: {:.4} em (negative = below baseline)", denominator.y);
            println!("    content: {:?}", element_type(&denominator.element));
            println!();
            if let Some(bar) = bar {
                println!("  bar:");
                println!("    thickness: {:.4} em", bar.thickness);
                println!("    style: {:?}", bar.style);
            }
        }
    } else {
        println!("(Fraction not found in semantic mode - check if semantic_mode is enabled)");
    }
    println!();
}

fn demo_scripts(options: &Options) {
    println!("--- Demo 3: Superscript/Subscript ---");
    println!("Input: x^2_i\n");

    let tree = parse_tree(r"x^2_i", ParserConfig::default()).unwrap();
    let layout = ir::build_ir(&tree, options);

    // Find the Scripts element
    for item in layout.walk() {
        if let ir::MathElement::Scripts { base, superscript, subscript, .. } = item.element {
            println!("Scripts Structure:");
            if let Some(base) = base {
                println!("  base: at ({:.4}, {:.4})", base.x, base.y);
            }
            if let Some(sup) = superscript {
                println!("  superscript:");
                println!("    position: ({:.4}, {:.4})", sup.x, sup.y);
                println!("    y > 0 means above baseline (raised)");
            }
            if let Some(sub) = subscript {
                println!("  subscript:");
                println!("    position: ({:.4}, {:.4})", sub.x, sub.y);
                println!("    y < 0 means below baseline (lowered)");
            }
            break;
        }
    }
    println!();
}

fn demo_sqrt(options: &Options) {
    println!("--- Demo 4: Square Root ---");
    println!("Input: \\sqrt{{x}}\n");

    let tree = parse_tree(r"\sqrt{x}", ParserConfig::default()).unwrap();
    let layout = ir::build_ir(&tree, options);

    // Find the Radical element
    let radical = layout.walk().find(|item| {
        matches!(item.element, ir::MathElement::Radical { .. })
    });

    if let Some(rad_item) = radical {
        if let ir::MathElement::Radical { radicand, index, .. } = rad_item.element {
            println!("Radical Structure:");
            println!("  radicand: {:?}", element_type(radicand));
            println!("  index: {:?}", index.as_ref().map(|i| element_type(i)));
            println!();
            println!("Layout dimensions: {:.4} x {:.4} em", layout.width, layout.height);
            println!();
            println!("Layout includes:");
            println!("  - Surd path (the radical symbol)");
            println!("  - Vinculum (horizontal rule above radicand)");
            println!("  - Proper clearance between content and vinculum");
        }
    }
    println!();

    // With index
    println!("Input: \\sqrt[3]{{x}}\n");
    let tree = parse_tree(r"\sqrt[3]{x}", ParserConfig::default()).unwrap();
    let layout = ir::build_ir(&tree, options);

    let radical = layout.walk().find(|item| {
        matches!(item.element, ir::MathElement::Radical { .. })
    });

    if let Some(rad_item) = radical {
        if let ir::MathElement::Radical { index, .. } = rad_item.element {
            if index.is_some() {
                println!("With index: index is positioned to upper-left of surd");
                println!("Layout dimensions: {:.4} x {:.4} em", layout.width, layout.height);
            }
        }
    }
    println!();
}

fn demo_spacing(options: &Options) {
    println!("--- Demo 5: Inter-Element Spacing ---");
    println!("Input: a + b = c\n");

    let tree = parse_tree("a + b = c", ParserConfig::default()).unwrap();
    let layout = ir::build_ir(&tree, options);

    println!("Spacing is automatically inserted based on atom types:");
    println!("  - Ord + Bin (a +): medium space (4mu = 4/18 em)");
    println!("  - Bin + Ord (+ b): medium space");
    println!("  - Ord + Rel (b =): thick space (5mu = 5/18 em)");
    println!("  - Rel + Ord (= c): thick space");
    println!();
    println!("Total width: {:.4} em", layout.width);
    println!("(includes spacing between atoms)");
    println!();

    // Show tight spacing in scripts
    println!("In script/scriptscript styles, spacing is tighter:");
    println!("  - Only Ord-Op and Op-Ord get thin space (3mu)");
    println!("  - Other combinations get no automatic spacing");
    println!();
}

fn demo_walker(options: &Options) {
    println!("--- Demo 6: Walking the Tree ---");
    println!("Input: \\frac{{x^2}}{{y}}\n");

    let tree = parse_tree(r"\frac{x^2}{y}", ParserConfig::default()).unwrap();
    let layout = ir::build_ir(&tree, options);

    println!("Walking all elements with absolute positions:");
    println!();

    for item in layout.walk() {
        let type_name = element_type(item.element);
        // Only show leaf elements for clarity
        match item.element {
            ir::MathElement::Text { text, style } => {
                println!("  Text '{}':", text);
                println!("    absolute position: ({:.4}, {:.4})", item.abs_x, item.abs_y);
                println!("    size multiplier: {:.2}", style.size);
                if let Some(font) = &style.font {
                    println!("    font: {:?}", font);
                }
                if let Some(w) = style.width {
                    println!("    width: {:.4} em", w);
                }
            }
            ir::MathElement::Kern { width } if *width != 0.0 => {
                println!("  Kern: {:.4} em at x={:.4}", width, item.abs_x);
            }
            ir::MathElement::Rule { width, height, .. } => {
                println!("  Rule: {:.4} x {:.4} em at ({:.4}, {:.4})",
                    width, height, item.abs_x, item.abs_y);
            }
            _ => {
                // Skip container elements in output
            }
        }
    }
    println!();
}

fn demo_layout_only(options: &Options) {
    println!("--- Demo 7: Layout-Only Mode ---");
    println!("Input: \\frac{{1}}{{2}}\n");

    let tree = parse_tree(r"\frac{1}{2}", ParserConfig::default()).unwrap();

    // Use layout-only config (no semantic variants)
    let config = ir::IrBuilderConfig::layout_only();
    let layout = ir::build_ir_with_config(&tree, options, &config);

    println!("In layout-only mode, semantic variants are not emitted.");
    println!("Instead of MathElement::Fraction, you get MathElement::VBox");
    println!();
    println!("Root element type: {:?}", element_type(&layout.root));
    println!();
    println!("Use layout-only mode when you only need positioning info");
    println!("and don't need to know the semantic structure.");
    println!();
}

/// Helper to get element type name
fn element_type(elem: &ir::MathElement) -> &'static str {
    match elem {
        ir::MathElement::Text { .. } => "Text",
        ir::MathElement::HBox { .. } => "HBox",
        ir::MathElement::VBox { .. } => "VBox",
        ir::MathElement::Rule { .. } => "Rule",
        ir::MathElement::Path { .. } => "Path",
        ir::MathElement::Kern { .. } => "Kern",
        ir::MathElement::Phantom { .. } => "Phantom",
        ir::MathElement::Color { .. } => "Color",
        ir::MathElement::Link { .. } => "Link",
        ir::MathElement::Image { .. } => "Image",
        ir::MathElement::Breakable { .. } => "Breakable",
        ir::MathElement::Fraction { .. } => "Fraction",
        ir::MathElement::Scripts { .. } => "Scripts",
        ir::MathElement::Radical { .. } => "Radical",
        ir::MathElement::Accent { .. } => "Accent",
        ir::MathElement::Delimited { .. } => "Delimited",
        ir::MathElement::LargeOp { .. } => "LargeOp",
        ir::MathElement::Array { .. } => "Array",
    }
}
