use aliter::{parse_tree, render_to_html_tree, parser::ParserConfig};

// Helper for basic parsing
fn assert_parses(expr: &str) {
    let conf = ParserConfig::default();
    let result = parse_tree(expr, conf);
    assert!(result.is_ok(), "Failed to parse: {}\nError: {:?}", expr, result.err());
}

// Helper with display mode enabled
fn assert_parses_display(expr: &str) {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;
    let result = parse_tree(expr, conf);
    assert!(result.is_ok(), "Failed to parse: {}\nError: {:?}", expr, result.err());
}

// Helper for rendering
fn assert_builds(expr: &str) {
    let conf = ParserConfig::default();
    let _tree = render_to_html_tree(expr, conf);
}

// Helper for rendering in display mode
fn assert_builds_display(expr: &str) {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;
    let _tree = render_to_html_tree(expr, conf);
}

// Helper that expects parsing to fail
fn assert_fails(expr: &str) {
    let conf = ParserConfig::default();
    let result = parse_tree(expr, conf);
    assert!(result.is_err(), "Expected parse error for: {}", expr);
}

// =============================================================================
// Matrix Environment Tests
// =============================================================================

#[test]
fn should_parse_matrix() {
    assert_parses(r"\begin{matrix}a&b\\c&d\end{matrix}");
}

#[test]
fn should_parse_pmatrix() {
    assert_parses(r"\begin{pmatrix}a&b\\c&d\end{pmatrix}");
}

#[test]
fn should_parse_bmatrix() {
    assert_parses(r"\begin{bmatrix}a&b\\c&d\end{bmatrix}");
}

#[test]
fn should_parse_vmatrix() {
    assert_parses(r"\begin{vmatrix}a&b\\c&d\end{vmatrix}");
}

#[test]
fn should_parse_uppercase_vmatrix() {
    assert_parses(r"\begin{Vmatrix}a&b\\c&d\end{Vmatrix}");
}

#[test]
fn should_parse_bbraces_matrix() {
    assert_parses(r"\begin{Bmatrix}a&b\\c&d\end{Bmatrix}");
}

#[test]
fn should_parse_larger_matrices() {
    assert_parses(r"\begin{pmatrix}1&2&3\\4&5&6\\7&8&9\end{pmatrix}");
}

#[test]
fn should_parse_single_row_matrix() {
    assert_parses(r"\begin{pmatrix}a&b&c\end{pmatrix}");
}

#[test]
fn should_parse_single_column_matrix() {
    assert_parses(r"\begin{pmatrix}a\\b\\c\end{pmatrix}");
}

#[test]
fn should_parse_1x1_matrix() {
    assert_parses(r"\begin{pmatrix}x\end{pmatrix}");
}

// =============================================================================
// Array Environment Tests
// =============================================================================

#[test]
fn should_parse_array_with_alignment() {
    assert_parses(r"\begin{array}{cc}a&b\\c&d\end{array}");
}

#[test]
fn should_parse_array_with_mixed_alignment() {
    assert_parses(r"\begin{array}{lcr}a&b&c\\d&e&f\end{array}");
}

#[test]
fn should_parse_array_with_vertical_lines() {
    assert_parses(r"\begin{array}{c|c}a&b\\c&d\end{array}");
}

#[test]
fn should_parse_array_with_multiple_vertical_lines() {
    assert_parses(r"\begin{array}{|c|c|}a&b\\c&d\end{array}");
}

#[test]
fn should_parse_array_with_hline() {
    assert_parses(r"\begin{array}{cc}a&b\\\hline c&d\end{array}");
}

// =============================================================================
// Cases Environment Tests
// =============================================================================

#[test]
fn should_parse_cases() {
    assert_parses(r"f(x)=\begin{cases}x&\text{if }x>0\\-x&\text{if }x<0\end{cases}");
}

#[test]
fn should_parse_cases_multiline() {
    assert_parses(r"\begin{cases}a&b\\c&d\\e&f\end{cases}");
}

#[test]
fn should_parse_dcases() {
    assert_parses(r"\begin{dcases}x&\text{if }x>0\\-x&\text{if }x<0\end{dcases}");
}

#[test]
fn should_parse_rcases() {
    assert_parses(r"\begin{rcases}a\\b\end{rcases}=c");
}

#[test]
fn should_parse_drcases() {
    assert_parses(r"\begin{drcases}a\\b\end{drcases}=c");
}

// =============================================================================
// Align Environment Tests (Display Mode Required)
// =============================================================================

#[test]
fn should_parse_aligned() {
    assert_parses(r"\begin{aligned}a&=b\\c&=d\end{aligned}");
}

#[test]
fn should_parse_align_in_display_mode() {
    assert_parses_display(r"\begin{align}a&=b\\c&=d\end{align}");
}

#[test]
fn should_parse_align_star() {
    assert_parses_display(r"\begin{align*}a&=b\\c&=d\end{align*}");
}

#[test]
fn should_parse_alignat() {
    assert_parses_display(r"\begin{alignat}{2}a&=b&c&=d\\e&=f&g&=h\end{alignat}");
}

#[test]
fn should_parse_alignedat() {
    assert_parses(r"\begin{alignedat}{2}a&=b&c&=d\\e&=f&g&=h\end{alignedat}");
}

// =============================================================================
// Gather Environment Tests
// =============================================================================

#[test]
fn should_parse_gathered() {
    assert_parses(r"\begin{gathered}a+b\\c+d\end{gathered}");
}

#[test]
fn should_parse_gather_in_display_mode() {
    assert_parses_display(r"\begin{gather}a+b\\c+d\end{gather}");
}

#[test]
fn should_parse_gather_star() {
    assert_parses_display(r"\begin{gather*}a+b\\c+d\end{gather*}");
}

// =============================================================================
// Split Environment Tests
// =============================================================================

#[test]
fn should_parse_split() {
    assert_parses(r"\begin{split}a&=b+c\\&=d\end{split}");
}

// =============================================================================
// Multline Environment Tests
// =============================================================================

#[test]
fn should_parse_multline_in_display_mode() {
    assert_parses_display(r"\begin{multline}a+b+c\\+d+e+f\end{multline}");
}

#[test]
fn should_parse_multline_star() {
    assert_parses_display(r"\begin{multline*}a+b\\+c+d\end{multline*}");
}

// =============================================================================
// Small Matrix Tests
// =============================================================================

#[test]
fn should_parse_smallmatrix() {
    assert_parses(r"\begin{smallmatrix}a&b\\c&d\end{smallmatrix}");
}

// =============================================================================
// Equation Environment Tests
// =============================================================================

#[test]
fn should_parse_equation_in_display_mode() {
    assert_parses_display(r"\begin{equation}E=mc^2\end{equation}");
}

#[test]
fn should_parse_equation_star() {
    assert_parses_display(r"\begin{equation*}E=mc^2\end{equation*}");
}

// =============================================================================
// CD (Commutative Diagram) Environment Tests
// =============================================================================

#[test]
fn should_parse_cd_environment() {
    assert_parses(r"\begin{CD}A@>>>B\\@VVV@VVV\\C@>>>D\end{CD}");
}

// =============================================================================
// Build Tests (Rendering)
// =============================================================================

#[test]
fn should_build_matrices() {
    assert_builds(r"\begin{pmatrix}1&0\\0&1\end{pmatrix}");
    assert_builds(r"\begin{bmatrix}a&b\\c&d\end{bmatrix}");
}

#[test]
fn should_build_cases() {
    assert_builds(r"|x|=\begin{cases}x&x\geq 0\\-x&x<0\end{cases}");
}

#[test]
fn should_build_aligned() {
    assert_builds(r"\begin{aligned}x&=1\\y&=2\end{aligned}");
}

#[test]
fn should_build_array() {
    assert_builds(r"\begin{array}{cc}a&b\\c&d\end{array}");
}

// =============================================================================
// Nested Environment Tests
// =============================================================================

#[test]
fn should_parse_nested_matrices() {
    assert_parses(r"\begin{pmatrix}1&\begin{pmatrix}a\\b\end{pmatrix}\\2&3\end{pmatrix}");
}

#[test]
fn should_parse_matrix_in_cases() {
    assert_parses(r"\begin{cases}\begin{pmatrix}1\\0\end{pmatrix}&\text{case 1}\\\begin{pmatrix}0\\1\end{pmatrix}&\text{case 2}\end{cases}");
}

// =============================================================================
// Complex Content in Environments
// =============================================================================

#[test]
fn should_parse_fractions_in_matrix() {
    assert_parses(r"\begin{pmatrix}\frac{1}{2}&\frac{3}{4}\\\frac{5}{6}&\frac{7}{8}\end{pmatrix}");
}

#[test]
fn should_parse_roots_in_matrix() {
    assert_parses(r"\begin{pmatrix}\sqrt{2}&\sqrt{3}\\\sqrt{5}&\sqrt{7}\end{pmatrix}");
}

#[test]
fn should_parse_greek_letters_in_matrix() {
    assert_parses(r"\begin{pmatrix}\alpha&\beta\\\gamma&\delta\end{pmatrix}");
}

#[test]
fn should_parse_superscripts_in_matrix() {
    assert_parses(r"\begin{pmatrix}x^2&y^2\\z^2&w^2\end{pmatrix}");
}

// =============================================================================
// Environment with Operators
// =============================================================================

#[test]
fn should_parse_determinant() {
    assert_parses(r"\det\begin{vmatrix}a&b\\c&d\end{vmatrix}");
}

#[test]
fn should_parse_matrix_multiplication() {
    assert_parses(r"\begin{pmatrix}a&b\end{pmatrix}\begin{pmatrix}c\\d\end{pmatrix}");
}

// =============================================================================
// Alignment Features
// =============================================================================

#[test]
fn should_parse_multiple_alignment_points() {
    assert_parses(r"\begin{aligned}a&=b&c&=d\\e&=f&g&=h\end{aligned}");
}

#[test]
fn should_parse_equations_with_text() {
    assert_parses(r"\begin{aligned}x&=1&\text{(first)}\\y&=2&\text{(second)}\end{aligned}");
}

// =============================================================================
// Environment Error Cases
// =============================================================================

#[test]
fn should_fail_on_mismatched_environment() {
    assert_fails(r"\begin{pmatrix}a&b\end{bmatrix}");
}

#[test]
fn should_fail_on_unclosed_environment() {
    assert_fails(r"\begin{pmatrix}a&b");
}

// =============================================================================
// Special Matrix Features
// =============================================================================

#[test]
fn should_parse_empty_matrix_cells() {
    assert_parses(r"\begin{pmatrix}a&&c\\&b&\end{pmatrix}");
}

#[test]
fn should_parse_matrix_with_dots() {
    assert_parses(r"\begin{pmatrix}a_{11}&\cdots&a_{1n}\\\vdots&\ddots&\vdots\\a_{m1}&\cdots&a_{mn}\end{pmatrix}");
}

// =============================================================================
// Real-World Examples
// =============================================================================

#[test]
fn should_parse_identity_matrix() {
    assert_parses(r"I=\begin{pmatrix}1&0&0\\0&1&0\\0&0&1\end{pmatrix}");
}

#[test]
fn should_parse_rotation_matrix() {
    assert_parses(r"R=\begin{pmatrix}\cos\theta&-\sin\theta\\\sin\theta&\cos\theta\end{pmatrix}");
}

#[test]
fn should_parse_system_of_equations() {
    assert_parses(r"\begin{aligned}2x+3y&=5\\4x-y&=7\end{aligned}");
}

#[test]
fn should_parse_piecewise_function() {
    assert_parses(r"f(x)=\begin{cases}0&x<0\\x^2&0\leq x\leq 1\\1&x>1\end{cases}");
}

#[test]
fn should_build_complex_equation() {
    assert_builds_display(r"\begin{align}f(x)&=ax^2+bx+c\\&=(x-r_1)(x-r_2)\end{align}");
}

// =============================================================================
// Array with Complex Formatting
// =============================================================================

#[test]
fn should_parse_array_as_table() {
    assert_parses(r"\begin{array}{|l|c|r|}\hline\text{Left}&\text{Center}&\text{Right}\\\hline a&b&c\\\hline\end{array}");
}

#[test]
fn should_parse_array_with_paragraph_column() {
    assert_parses(r"\begin{array}{p{2em}}\text{paragraph}\\\text{text}\end{array}");
}
