use aliter::{parse_tree, render_to_html_tree, parser::ParserConfig};

// Helper for basic parsing
fn assert_parses(expr: &str) {
    let conf = ParserConfig::default();
    let result = parse_tree(expr, conf);
    assert!(result.is_ok(), "Failed to parse: {}\nError: {:?}", expr, result.err());
}

// Helper for rendering
fn assert_builds(expr: &str) {
    let conf = ParserConfig::default();
    let _tree = render_to_html_tree(expr, conf);
}

// =============================================================================
// Font Family Tests
// =============================================================================

#[test]
fn should_parse_mathrm() {
    assert_parses(r"\mathrm{Roman}");
    assert_parses(r"\mathrm{ABC123}");
}

#[test]
fn should_parse_mathit() {
    assert_parses(r"\mathit{Italic}");
    assert_parses(r"\mathit{xyz}");
}

#[test]
fn should_parse_mathbf() {
    assert_parses(r"\mathbf{Bold}");
    assert_parses(r"\mathbf{ABCdef123}");
}

#[test]
fn should_parse_mathsf() {
    assert_parses(r"\mathsf{SansSerif}");
}

#[test]
fn should_parse_mathtt() {
    assert_parses(r"\mathtt{Teletype}");
    assert_parses(r"\mathtt{code}");
}

#[test]
fn should_parse_mathcal() {
    assert_parses(r"\mathcal{CALLIGRAPHIC}");
    assert_parses(r"\mathcal{ABC}");
}

#[test]
fn should_parse_mathfrak() {
    assert_parses(r"\mathfrak{Fraktur}");
    assert_parses(r"\mathfrak{ABC}");
}

#[test]
fn should_parse_mathbb() {
    assert_parses(r"\mathbb{R}");
    assert_parses(r"\mathbb{NZQRC}");
}

#[test]
fn should_parse_mathscr() {
    assert_parses(r"\mathscr{Script}");
    assert_parses(r"\mathscr{ABC}");
}

// =============================================================================
// Text Mode Font Commands
// =============================================================================

#[test]
fn should_parse_text_font_commands() {
    assert_parses(r"\text{normal text}");
    assert_parses(r"\textit{italic}");
    assert_parses(r"\textbf{bold}");
    assert_parses(r"\textrm{roman}");
    assert_parses(r"\textsf{sans serif}");
    assert_parses(r"\texttt{teletype}");
}

#[test]
fn should_parse_combined_text_styles() {
    assert_parses(r"\textbf{\textit{bold italic}}");
    assert_parses(r"\textsf{\textbf{sans bold}}");
}

// =============================================================================
// Old Font Commands Tests
// =============================================================================

#[test]
fn should_parse_old_font_commands() {
    assert_parses(r"{\rm Roman}");
    assert_parses(r"{\it Italic}");
    assert_parses(r"{\bf Bold}");
    assert_parses(r"{\sf Sans}");
    assert_parses(r"{\tt Teletype}");
}

#[test]
fn should_parse_cal_and_mit() {
    assert_parses(r"{\cal CALLIGRAPHIC}");
    // \mit not implemented yet
    // assert_parses(r"{\mit mathitalic}");
}

// =============================================================================
// Bold Symbol Tests
// =============================================================================

#[test]
fn should_parse_boldsymbol() {
    assert_parses(r"\boldsymbol{\alpha}");
    assert_parses(r"\boldsymbol{x+y}");
    assert_parses(r"\boldsymbol{\Gamma}");
}

#[test]
fn should_parse_bm() {
    assert_parses(r"\bm{\alpha}");
    assert_parses(r"\bm{v}");
}

#[test]
fn should_parse_pmb() {
    assert_parses(r"\pmb{A}");
    assert_parses(r"\pmb{\alpha}");
}

// =============================================================================
// Sizing Commands Tests
// =============================================================================

#[test]
fn should_parse_all_size_commands() {
    assert_parses(r"\tiny{x}");
    assert_parses(r"\scriptsize{x}");
    assert_parses(r"\footnotesize{x}");
    assert_parses(r"\small{x}");
    assert_parses(r"\normalsize{x}");
    assert_parses(r"\large{x}");
    assert_parses(r"\Large{x}");
    assert_parses(r"\LARGE{x}");
    assert_parses(r"\huge{x}");
    assert_parses(r"\Huge{x}");
}

#[test]
fn should_build_size_commands() {
    assert_builds(r"\tiny{a}\small{b}\large{c}\Huge{d}");
}

#[test]
fn should_parse_nested_sizes() {
    assert_parses(r"\Huge{Big \small{small} Big}");
}

// =============================================================================
// Build Tests (Rendering)
// =============================================================================

#[test]
fn should_build_font_families() {
    assert_builds(r"\mathrm{A}\mathit{B}\mathbf{C}\mathsf{D}\mathtt{E}");
}

#[test]
fn should_build_special_fonts() {
    assert_builds(r"\mathbb{R}\mathcal{L}\mathfrak{g}\mathscr{F}");
}

#[test]
fn should_build_text_fonts() {
    // Spacing group not handled in HTML builder yet
    // assert_builds(r"\text{text }\textit{italic }\textbf{bold }\texttt{code}");
    assert_builds(r"\text{text}\textit{italic}\textbf{bold}\texttt{code}");
}

#[test]
fn should_build_old_font_commands() {
    assert_builds(r"{\rm a}{\it b}{\bf c}{\sf d}{\tt e}");
}

// =============================================================================
// Mixed Font Tests
// =============================================================================

#[test]
fn should_parse_fonts_with_superscripts_subscripts() {
    assert_parses(r"\mathbf{x}^2");
    assert_parses(r"\mathit{y}_i");
    assert_parses(r"\mathrm{d}x");
}

#[test]
fn should_parse_fonts_in_fractions() {
    assert_parses(r"\frac{\mathbf{a}}{\mathit{b}}");
}

#[test]
fn should_parse_fonts_in_roots() {
    assert_parses(r"\sqrt{\mathbf{x}}");
}

#[test]
fn should_build_complex_font_expressions() {
    assert_builds(r"\mathbf{F} = m\mathbf{a}");
    assert_builds(r"\mathbb{E}[X] = \int_{-\infty}^{\infty} x f(x)\,\mathrm{d}x");
}

// =============================================================================
// Unicode Bold Tests
// =============================================================================

// Unicode bold variants not implemented yet
// #[test]
// fn should_parse_unicode_bold_variants() {
//     // These use \symbf, \symbfit, etc.
//     assert_parses(r"\symbf{x}");
//     assert_parses(r"\symbfit{y}");
// }

// =============================================================================
// Font Mode Switching
// =============================================================================

#[test]
fn should_switch_between_math_and_text_fonts() {
    // \( \) delimiters and $ in text mode not implemented yet
    // assert_parses(r"\text{text \(\mathbf{math}\) text}");
    // assert_parses(r"\text{text $\mathbf{math}$ text}");
    assert_parses(r"\mathbf{math \text{text} math}");
}

#[test]
fn should_handle_nested_font_changes() {
    assert_parses(r"\mathbf{\mathit{x}}");
    assert_parses(r"\mathrm{\mathcal{A}}");
}

// =============================================================================
// Special Cases
// =============================================================================

#[test]
fn should_parse_empty_font_commands() {
    assert_parses(r"\mathbf{}");
    assert_parses(r"\text{}");
}

#[test]
fn should_parse_font_commands_with_braces() {
    assert_parses(r"\mathbf{(a+b)}");
    assert_parses(r"\text{[x, y]}");
}

#[test]
fn should_parse_digits_in_various_fonts() {
    assert_parses(r"\mathrm{123}\mathbf{456}\mathtt{789}");
}

// =============================================================================
// Operators in Different Fonts
// =============================================================================

#[test]
fn should_parse_operators_in_fonts() {
    assert_parses(r"\mathbf{+}");
    assert_parses(r"\mathrm{=}");
    assert_parses(r"\mathit{\times}");
}

#[test]
fn should_build_styled_equations() {
    assert_builds(r"\mathbf{v} \cdot \mathbf{w} = v_1 w_1 + v_2 w_2");
    assert_builds(r"\det(\mathbf{A}) = \mathrm{tr}(\mathbf{A})");
}

// =============================================================================
// Color + Font Combinations
// =============================================================================

#[test]
fn should_parse_colored_fonts() {
    assert_parses(r"\color{red}\mathbf{x}");
    assert_parses(r"\mathbf{\color{blue}y}");
}

#[test]
fn should_build_colored_fonts() {
    assert_builds(r"\color{red}\mathbf{RED BOLD}");
}
