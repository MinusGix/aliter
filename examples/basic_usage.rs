//! Basic usage examples for aliter - a Rust LaTeX math renderer.
//!
//! Run with: `cargo run --example basic_usage`

use aliter::{
    dom_tree::HtmlNode,
    ir::{from_html, to_html, MathElement, MathLayout},
    parse_tree,
    parser::ParserConfig,
    render_to_html_tree,
    tree::VirtualNode,
};

fn main() {
    println!("=== Aliter Usage Examples ===\n");

    // Example 1: Parse LaTeX to HTML string
    example_html_output();

    // Example 2: Parse and inspect the parse tree
    example_parse_tree();

    // Example 3: Use the IR for custom rendering
    example_ir_usage();

    // Example 4: Walk the IR tree
    example_ir_walker();

    // Example 5: Display mode vs inline mode
    example_display_mode();
}

/// Render LaTeX to an HTML string
fn example_html_output() {
    println!("--- Example 1: HTML Output ---");

    let latex = r"\frac{-b \pm \sqrt{b^2 - 4ac}}{2a}";
    let conf = ParserConfig::default();

    // Render to HTML tree, then convert to string
    let html_tree = render_to_html_tree(latex, conf);
    let html_string = html_tree.to_markup();

    println!("LaTeX: {}", latex);
    println!("HTML length: {} chars", html_string.len());
    println!("HTML preview: {}...\n", &html_string[..html_string.len().min(200)]);
}

/// Parse LaTeX and inspect the parse tree
fn example_parse_tree() {
    println!("--- Example 2: Parse Tree ---");

    let latex = r"x^2 + y^2 = z^2";
    let conf = ParserConfig::default();

    match parse_tree(latex, conf) {
        Ok(tree) => {
            println!("LaTeX: {}", latex);
            println!("Parse tree has {} top-level nodes", tree.len());
            for (i, node) in tree.iter().enumerate() {
                println!("  Node {}: {:?}", i, node.typ());
            }
        }
        Err(e) => {
            println!("Parse error: {:?}", e);
        }
    }
    println!();
}

/// Use the IR (Intermediate Representation) for custom rendering
fn example_ir_usage() {
    println!("--- Example 3: IR for Custom Rendering ---");

    let latex = r"\sum_{i=1}^{n} i^2";
    let conf = ParserConfig::default();

    // Step 1: Render to HTML tree
    let html_tree = render_to_html_tree(latex, conf);

    // Step 2: Convert to IR
    let ir = from_html::convert(&HtmlNode::Span(html_tree));

    // Step 3: Create layout with dimensions
    let layout = MathLayout::new(ir, false);

    println!("LaTeX: {}", latex);
    println!("Layout dimensions:");
    println!("  Width:  {:.3} em", layout.width);
    println!("  Height: {:.3} em (above baseline)", layout.height);
    println!("  Depth:  {:.3} em (below baseline)", layout.depth);

    // Step 4: Optionally render IR back to HTML
    let html_from_ir = to_html::render(&layout);
    println!("Re-rendered HTML length: {} chars\n", html_from_ir.len());
}

/// Walk the IR tree and extract rendering primitives
fn example_ir_walker() {
    println!("--- Example 4: IR Walker ---");

    let latex = r"E = mc^2";
    let conf = ParserConfig::default();

    let html_tree = render_to_html_tree(latex, conf);
    let ir = from_html::convert(&HtmlNode::Span(html_tree));
    let layout = MathLayout::new(ir, false);

    println!("LaTeX: {}", latex);
    println!("Walking IR tree:");

    let mut text_count = 0;
    let mut box_count = 0;
    let mut kern_count = 0;

    for item in layout.walk() {
        match item.element {
            MathElement::Text { ref text, ref style } => {
                text_count += 1;
                let font_name = style
                    .font
                    .as_ref()
                    .map(|f| f.as_str())
                    .unwrap_or("default");
                println!(
                    "  TEXT '{}' at ({:.2}, {:.2}) font={}",
                    text, item.abs_x, item.abs_y, font_name
                );
            }
            MathElement::HBox { .. } => box_count += 1,
            MathElement::VBox { .. } => box_count += 1,
            MathElement::Kern { width } => {
                if *width != 0.0 {
                    kern_count += 1;
                }
            }
            MathElement::Rule { width, height, .. } => {
                println!(
                    "  RULE at ({:.2}, {:.2}) size={:.2}x{:.2}",
                    item.abs_x, item.abs_y, width, height
                );
            }
            _ => {}
        }
    }

    println!("Summary: {} text, {} boxes, {} kerns\n", text_count, box_count, kern_count);
}

/// Compare display mode vs inline mode
fn example_display_mode() {
    println!("--- Example 5: Display Mode ---");

    let latex = r"\int_0^\infty e^{-x^2} dx";

    // Inline mode (default)
    let mut conf_inline = ParserConfig::default();
    conf_inline.display_mode = false;
    let html_inline = render_to_html_tree(latex, conf_inline);

    // Display mode
    let mut conf_display = ParserConfig::default();
    conf_display.display_mode = true;
    let html_display = render_to_html_tree(latex, conf_display);

    println!("LaTeX: {}", latex);
    println!("Inline mode height:  {:.3} em", html_inline.node.height);
    println!("Display mode height: {:.3} em", html_display.node.height);
    println!("(Display mode typically renders larger)\n");
}
