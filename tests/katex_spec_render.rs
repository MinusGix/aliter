use aliter::{parse_tree, parser::ParserConfig, render_to_html_tree, tree::VirtualNode};

// Basic render smoke tests mirroring a subset of KaTeX katex-spec.js
// These are meant to surface builder/serialization regressions quickly.

fn render(expr: &str) -> String {
    let conf = ParserConfig::default();
    let tree = render_to_html_tree(expr, conf);
    tree.to_markup()
}

#[test]
fn render_empty_and_simple_ord() {
    let conf = ParserConfig::default();
    assert!(parse_tree("", conf.clone()).is_ok());
    let html = render("x");
    assert!(
        html.contains("katex-html") || html.contains("katex"),
        "expected katex wrapper in {html}"
    );
}

#[test]
fn render_supsub() {
    let html = render("x^2_3");
    assert!(
        html.contains("sup") || html.contains("sub"),
        "expected superscript/subscript structure in {html}"
    );
}

#[test]
fn render_frac() {
    let html = render("\\frac{1}{2}");
    assert!(
        html.contains("frac") || html.contains("mfrac"),
        "expected fraction structure in {html}"
    );
}

#[test]
fn render_sqrt() {
    let html = render("\\sqrt{2}");
    assert!(
        html.contains("sqrt") || html.contains("msqrt"),
        "expected sqrt structure in {html}"
    );
}

#[test]
fn render_color() {
    let html = render("\\textcolor{#fff}{x}");
    assert!(
        html.contains("color") || html.contains("style=\"color"),
        "expected color styling in {html}"
    );
}

#[test]
fn render_text_mode() {
    let html = render("\\text{abc}");
    assert!(
        html.contains("text") || html.contains("mtext"),
        "expected text nodes in {html}"
    );
}

#[test]
fn render_over_under() {
    let html = render("\\overline{x} + \\underline{y}");
    assert!(
        html.contains("overline") || html.contains("underline") || html.contains("mover"),
        "expected over/underline markers in {html}"
    );
}

#[test]
fn render_delims() {
    let html = render("\\left( x^2 \\right)");
    assert!(
        html.contains("(") && html.contains(")"),
        "expected delimiters in {html}"
    );
}

#[test]
fn render_rule() {
    let html = render("\\rule{1em}{2em}");
    assert!(
        html.contains("rule") || html.contains("span"),
        "expected rule span in {html}"
    );
}

#[test]
fn render_phantom() {
    let html = render("\\phantom{xyz}");
    assert!(
        html.contains("phantom") || html.contains("span"),
        "expected phantom span in {html}"
    );
}
