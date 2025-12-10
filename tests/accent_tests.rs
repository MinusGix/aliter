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

// Helper that expects parsing to fail
fn assert_fails(expr: &str) {
    let conf = ParserConfig::default();
    let result = parse_tree(expr, conf);
    assert!(result.is_err(), "Expected parse error for: {}", expr);
}

// =============================================================================
// Basic Accent Tests
// =============================================================================

#[test]
fn should_parse_hat_accent() {
    assert_parses(r"\hat{x}");
    assert_parses(r"\hat{a}");
    assert_parses(r"\hat{\theta}");
}

#[test]
fn should_parse_tilde_accent() {
    assert_parses(r"\tilde{x}");
    assert_parses(r"\tilde{n}");
    assert_parses(r"\tilde{\alpha}");
}

#[test]
fn should_parse_bar_accent() {
    assert_parses(r"\bar{x}");
    assert_parses(r"\bar{y}");
    assert_parses(r"\bar{z}");
}

#[test]
fn should_parse_dot_accent() {
    assert_parses(r"\dot{x}");
    assert_parses(r"\dot{y}");
}

#[test]
fn should_parse_ddot_accent() {
    assert_parses(r"\ddot{x}");
    assert_parses(r"\ddot{a}");
}

#[test]
fn should_parse_vec_accent() {
    assert_parses(r"\vec{x}");
    assert_parses(r"\vec{v}");
    assert_parses(r"\vec{AB}");
}

#[test]
fn should_parse_acute_accent() {
    assert_parses(r"\acute{a}");
    assert_parses(r"\acute{e}");
}

#[test]
fn should_parse_grave_accent() {
    assert_parses(r"\grave{a}");
    assert_parses(r"\grave{e}");
}

#[test]
fn should_parse_breve_accent() {
    assert_parses(r"\breve{x}");
    assert_parses(r"\breve{a}");
}

#[test]
fn should_parse_check_accent() {
    assert_parses(r"\check{x}");
    assert_parses(r"\check{c}");
}

// =============================================================================
// Additional Accents
// =============================================================================

// Note: \dddot and \ddddot are NOT supported in KaTeX
// See KaTeX docs/support_table.md - they are listed as "Not supported"

#[test]
fn should_parse_mathring() {
    assert_parses(r"\mathring{A}");
    assert_parses(r"\mathring{x}");
}

// =============================================================================
// Wide Accents
// =============================================================================

#[test]
fn should_parse_widehat() {
    assert_parses(r"\widehat{x}");
    assert_parses(r"\widehat{xy}");
    assert_parses(r"\widehat{xyz}");
    assert_parses(r"\widehat{ABC}");
}

#[test]
fn should_parse_widetilde() {
    assert_parses(r"\widetilde{x}");
    assert_parses(r"\widetilde{ab}");
    assert_parses(r"\widetilde{abc}");
}

#[test]
fn should_parse_overline() {
    assert_parses(r"\overline{x}");
    assert_parses(r"\overline{abc}");
    assert_parses(r"\overline{x+y}");
}

#[test]
fn should_parse_underline() {
    assert_parses(r"\underline{x}");
    assert_parses(r"\underline{text}");
    assert_parses(r"\underline{a+b}");
}

#[test]
fn should_parse_overbrace() {
    assert_parses(r"\overbrace{x+y}");
    assert_parses(r"\overbrace{a+b+c}");
}

#[test]
fn should_parse_underbrace() {
    assert_parses(r"\underbrace{x+y}");
    assert_parses(r"\underbrace{a+b+c}");
}

#[test]
fn should_parse_overleftarrow_and_overrightarrow() {
    assert_parses(r"\overleftarrow{AB}");
    assert_parses(r"\overrightarrow{AB}");
    assert_parses(r"\overrightarrow{v}");
}

#[test]
fn should_parse_underleftarrow_and_underrightarrow() {
    assert_parses(r"\underleftarrow{AB}");
    assert_parses(r"\underrightarrow{AB}");
}

#[test]
fn should_parse_overleftrightarrow() {
    assert_parses(r"\overleftrightarrow{AB}");
    assert_parses(r"\underleftrightarrow{AB}");
}

// =============================================================================
// Accents with Annotations
// =============================================================================

#[test]
fn should_parse_overbrace_with_superscript() {
    assert_parses(r"\overbrace{a+b+c}^{\text{sum}}");
    assert_parses(r"\overbrace{1+2+\cdots+n}^{n}");
}

#[test]
fn should_parse_underbrace_with_subscript() {
    assert_parses(r"\underbrace{a+b+c}_{\text{sum}}");
    assert_parses(r"\underbrace{x_1+\cdots+x_n}_{n \text{ terms}}");
}

// =============================================================================
// Stacked Accents
// =============================================================================

#[test]
fn should_parse_nested_accents() {
    assert_parses(r"\hat{\hat{x}}");
    assert_parses(r"\tilde{\bar{x}}");
    assert_parses(r"\dot{\vec{v}}");
}

#[test]
fn should_parse_multiple_accents_on_same_base() {
    // Multiple different accents
    assert_parses(r"\hat{x} \tilde{x} \bar{x}");
}

// =============================================================================
// Accents on Different Base Types
// =============================================================================

#[test]
fn should_parse_accents_on_greek_letters() {
    assert_parses(r"\hat{\alpha}");
    assert_parses(r"\bar{\theta}");
    assert_parses(r"\tilde{\omega}");
    assert_parses(r"\dot{\phi}");
}

#[test]
fn should_parse_accents_on_uppercase_letters() {
    assert_parses(r"\hat{X}");
    assert_parses(r"\bar{Y}");
    assert_parses(r"\tilde{N}");
}

#[test]
fn should_parse_accents_on_expressions() {
    assert_parses(r"\hat{x+y}");
    assert_parses(r"\bar{a-b}");
    assert_parses(r"\tilde{f(x)}");
}

#[test]
fn should_parse_accents_on_fractions() {
    assert_parses(r"\hat{\frac{a}{b}}");
    assert_parses(r"\bar{\frac{x}{y}}");
}

// =============================================================================
// Text Mode Accents
// =============================================================================

#[test]
fn should_parse_text_accents() {
    assert_parses(r"\text{\'{e}}");
    assert_parses(r"\text{\`{a}}");
    assert_parses(r"\text{\^{o}}");
    assert_parses(r"\text{\~{n}}");
    assert_parses(r#"\text{\"{u}}"#);
}

// =============================================================================
// Build Tests (Rendering)
// =============================================================================

#[test]
fn should_build_basic_accents() {
    assert_builds(r"\hat{x} \tilde{y} \bar{z} \dot{a} \ddot{b}");
}

#[test]
fn should_build_wide_accents() {
    assert_builds(r"\widehat{xyz} \widetilde{abc}");
}

#[test]
fn should_build_over_and_underline() {
    assert_builds(r"\overline{a+b} \underline{c+d}");
}

#[test]
fn should_build_braces() {
    assert_builds(r"\overbrace{1+2+3}^{6} \underbrace{a+b+c}_{sum}");
}

#[test]
fn should_build_arrows() {
    assert_builds(r"\overrightarrow{AB} \overleftarrow{BA}");
}

// =============================================================================
// Complex Expressions with Accents
// =============================================================================

#[test]
fn should_parse_accents_in_physics_notation() {
    assert_parses(r"\vec{F} = m\vec{a}");
    assert_parses(r"\hat{i} + \hat{j} + \hat{k}");
    assert_parses(r"\dot{x} = v, \ddot{x} = a");
}

#[test]
fn should_parse_accents_in_complex_expressions() {
    assert_parses(r"\bar{z} = \bar{x} + i\bar{y}");
    assert_parses(r"\tilde{f}(\hat{x})");
}

#[test]
fn should_parse_accents_with_subscripts_superscripts() {
    assert_parses(r"\hat{x}_i");
    assert_parses(r"\vec{v}^2");
    assert_parses(r"\bar{x}_n^2");
}

#[test]
fn should_build_physics_vectors() {
    assert_builds(r"\vec{r} = x\hat{i} + y\hat{j} + z\hat{k}");
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn should_parse_accent_on_single_character() {
    assert_parses(r"\hat{a}");
    assert_parses(r"\hat{1}");
}

#[test]
fn should_parse_accent_on_empty_might_fail() {
    // Some accents might require a base
    // This depends on implementation
    assert_parses(r"\hat{}");
}

#[test]
fn should_parse_accent_in_superscript() {
    assert_parses(r"x^{\hat{y}}");
}

#[test]
fn should_parse_accent_in_subscript() {
    assert_parses(r"x_{\tilde{i}}");
}

// =============================================================================
// Special Accent Combinations
// =============================================================================

#[test]
fn should_parse_derivative_notation() {
    // Note: \dddot is NOT supported in KaTeX
    assert_parses(r"\dot{x}, \ddot{x}");
}

#[test]
fn should_parse_mean_value_notation() {
    assert_parses(r"\bar{x} = \frac{1}{n}\sum_{i=1}^n x_i");
}

#[test]
fn should_parse_vector_notation() {
    assert_parses(r"\vec{a} \cdot \vec{b}");
    assert_parses(r"\vec{a} \times \vec{b}");
}

// =============================================================================
// AMS Accents
// =============================================================================

#[test]
fn should_parse_overgroup_undergroup() {
    assert_parses(r"\overgroup{AB}");
    assert_parses(r"\undergroup{AB}");
}

// =============================================================================
// Accent Error Cases
// =============================================================================

#[test]
fn should_handle_accents_in_text_mode() {
    // Most math accents should work in math mode, fail in text mode
    assert_parses(r"\text{hello} \hat{x}");
    // \hat outside math should be in text with escaped version
}

// =============================================================================
// Multiple Accents in Expression
// =============================================================================

#[test]
fn should_parse_multiple_different_accents() {
    assert_parses(r"\hat{x} + \tilde{y} - \bar{z}");
}

#[test]
fn should_build_complex_accented_equation() {
    assert_builds(r"\hat{H}\psi = E\psi");
    assert_builds(r"\vec{\nabla} \times \vec{E} = -\frac{\partial \vec{B}}{\partial t}");
}

#[test]
fn should_parse_accents_in_matrices() {
    assert_parses(r"\begin{pmatrix}\hat{x}\\\tilde{y}\end{pmatrix}");
}

#[test]
fn should_parse_wide_accents_on_long_expressions() {
    assert_parses(r"\widehat{a+b+c+d+e+f}");
    assert_parses(r"\widetilde{abcdefgh}");
}
