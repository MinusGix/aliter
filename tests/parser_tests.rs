use aliter::{parse_tree, parser::ParserConfig};

// Helper to check that parsing succeeds
fn assert_parses(expr: &str) {
    let conf = ParserConfig::default();
    let result = parse_tree(expr, conf);
    assert!(result.is_ok(), "Failed to parse: {}\nError: {:?}", expr, result.err());
}

// Helper with strict settings
fn assert_parses_strict(expr: &str) {
    let mut conf = ParserConfig::default();
    conf.strict = aliter::parser::StrictMode::Error;
    let result = parse_tree(expr, conf);
    assert!(result.is_ok(), "Failed to parse (strict): {}\nError: {:?}", expr, result.err());
}

// =============================================================================
// Basic Parser Tests (from KaTeX katex-spec.js)
// =============================================================================

#[test]
fn parser_should_not_fail_on_empty_string() {
    assert_parses_strict("");
}

#[test]
fn parser_should_ignore_whitespace() {
    // Both should parse successfully (we can't easily check AST equivalence yet)
    assert_parses_strict("    x    y    ");
    assert_parses_strict("xy");
}

#[test]
fn parser_should_ignore_whitespace_in_atom() {
    assert_parses_strict("    x   ^ y    ");
    assert_parses_strict("x^y");
}

// =============================================================================
// Ord Parser Tests
// =============================================================================

#[test]
fn ord_parser_should_not_fail() {
    let expression = "1234|/@.\"`abcdefgzABCDEFGZ";
    assert_parses(expression);
}

// TODO: Add test to check that all chars are parsed as ords
// This requires examining the parse tree structure

// =============================================================================
// Bin Parser Tests
// =============================================================================

#[test]
fn bin_parser_should_not_fail() {
    assert_parses(r"+-*\cdot\pm\div");
}

// TODO: Add test to verify all are parsed as bin atoms
// This requires examining the parse tree structure

// =============================================================================
// Rel Parser Tests
// =============================================================================

#[test]
fn rel_parser_should_not_fail() {
    assert_parses(r"=<>\leq\geq\neq\nleq\ngeq\cong");
    assert_parses(r"\not=\not<\not>\not\leq\not\geq\not\in");
}

// TODO: Add test to verify all are parsed as rel atoms
// This requires examining the parse tree structure

// =============================================================================
// Mathinner Parser Tests
// =============================================================================

#[test]
fn mathinner_parser_should_not_fail() {
    assert_parses(r"\mathinner{\langle{\psi}\rangle}");
    assert_parses(r"\frac 1 {\mathinner{\langle{\psi}\rangle}}");
}

// =============================================================================
// Punct Parser Tests
// =============================================================================

#[test]
fn punct_parser_should_not_fail() {
    assert_parses_strict(",;");
}

// TODO: Add test to verify they are parsed as punct atoms

// =============================================================================
// Open Parser Tests
// =============================================================================

#[test]
fn open_parser_should_not_fail() {
    assert_parses("([");
}

// TODO: Add test to verify they are parsed as open atoms

// =============================================================================
// Close Parser Tests
// =============================================================================

#[test]
fn close_parser_should_not_fail() {
    assert_parses(")]?!");
}

// TODO: Add test to verify they are parsed as close atoms

// =============================================================================
// KaTeX Command Parser Tests
// =============================================================================

#[test]
fn katex_parser_should_not_fail() {
    assert_parses(r"\KaTeX");
}

// =============================================================================
// Subscript and Superscript Parser Tests
// =============================================================================

#[test]
fn parser_should_not_fail_on_superscripts() {
    assert_parses("x^2");
}

#[test]
fn parser_should_not_fail_on_subscripts() {
    assert_parses("x_3");
}

#[test]
fn parser_should_not_fail_on_both_sub_and_super() {
    assert_parses("x^2_3");
    assert_parses("x_2^3");
}

#[test]
fn parser_should_not_fail_when_no_nucleus() {
    assert_parses("^3");
    assert_parses("^3+");
    assert_parses("_2");
    assert_parses("^3_2");
    assert_parses("_2^3");
}

// TODO: Add tests to verify supsub node structure in parse tree

// =============================================================================
// Limit Controls Tests
// =============================================================================

#[test]
fn parser_should_parse_limits_on_operators() {
    assert_parses(r"\int\limits_a^b");
    assert_parses(r"\sum\limits_{i=0}^n");
}

#[test]
fn parser_should_parse_nolimits_on_operators() {
    assert_parses(r"\int\nolimits_a^b");
}

// =============================================================================
// Group Parser Tests
// =============================================================================

#[test]
fn group_parser_should_not_fail() {
    assert_parses("{ x }");
    assert_parses("{x}{y}");
    assert_parses("{{x}}");
}

#[test]
fn begingroup_endgroup_parser_should_not_fail() {
    assert_parses(r"\begingroup x \endgroup");
}

// =============================================================================
// Function Parser Tests
// =============================================================================

#[test]
fn function_parser_should_not_fail_on_common_functions() {
    assert_parses(r"\sin");
    assert_parses(r"\cos x");
    assert_parses(r"\ln(x)");
    assert_parses(r"\log_2(x)");
    assert_parses(r"\lim_{x \to \infty}");
}

// =============================================================================
// Frac Parser Tests
// =============================================================================

#[test]
fn frac_parser_should_not_fail() {
    assert_parses(r"\frac{1}{2}");
    assert_parses(r"\frac{x}{y}");
    assert_parses(r"\dfrac{1}{2}");
    assert_parses(r"\tfrac{1}{2}");
    assert_parses(r"\cfrac{1}{2}");
}

// =============================================================================
// Over/Brace/Brack Parser Tests
// =============================================================================

#[test]
fn over_parser_should_not_fail() {
    assert_parses(r"1 \over 2");
}

#[test]
fn brace_parser_should_not_fail() {
    assert_parses(r"n \brace k");
}

#[test]
fn brack_parser_should_not_fail() {
    assert_parses(r"n \brack k");
}

// =============================================================================
// Sizing Parser Tests
// =============================================================================

#[test]
fn sizing_parser_should_not_fail() {
    assert_parses(r"\tiny x");
    assert_parses(r"\small x");
    assert_parses(r"\normalsize x");
    assert_parses(r"\large x");
    assert_parses(r"\Large x");
    assert_parses(r"\LARGE x");
    assert_parses(r"\huge x");
    assert_parses(r"\Huge x");
}

// =============================================================================
// Text Parser Tests
// =============================================================================

#[test]
fn text_parser_should_not_fail() {
    assert_parses(r"\text{hello world}");
    assert_parses(r"\text{hello $x$ world}");
}

#[test]
fn text_with_nested_math_should_work() {
    assert_parses(r"\text{The value is $x=5$}");
}

// =============================================================================
// Color Parser Tests
// =============================================================================

#[test]
fn color_parser_should_not_fail() {
    assert_parses(r"\textcolor{red}{x}");
    assert_parses(r"\textcolor{#ff0000}{x}");
    assert_parses(r"\textcolor{#f00}{x}");
    assert_parses(r"\color{blue}x");
}

#[test]
fn colorbox_parser_should_not_fail() {
    assert_parses(r"\colorbox{red}{x}");
    assert_parses(r"\fcolorbox{red}{yellow}{x}");
}

// =============================================================================
// Delimiter Sizing Parser Tests
// =============================================================================

#[test]
fn delimiter_sizing_should_not_fail() {
    assert_parses(r"\bigl( x \bigr)");
    assert_parses(r"\Bigl[ x \Bigr]");
    assert_parses(r"\biggl\{ x \biggr\}");
    assert_parses(r"\Biggl| x \Biggr|");
}

// =============================================================================
// Left/Right Parser Tests
// =============================================================================

#[test]
fn left_right_parser_should_not_fail() {
    assert_parses(r"\left( x \right)");
    assert_parses(r"\left[ x \right]");
    assert_parses(r"\left\{ x \right\}");
    assert_parses(r"\left| x \right|");
    assert_parses(r"\left. x \right|");
}

// =============================================================================
// Sqrt Parser Tests
// =============================================================================

#[test]
fn sqrt_parser_should_not_fail() {
    assert_parses(r"\sqrt{2}");
    assert_parses(r"\sqrt[3]{8}");
    assert_parses(r"\sqrt{1+\sqrt{2}}");
}

// =============================================================================
// Overline/Underline Parser Tests
// =============================================================================

#[test]
fn overline_parser_should_not_fail() {
    assert_parses(r"\overline{x}");
    assert_parses(r"\overline{AB}");
}

#[test]
fn underline_parser_should_not_fail() {
    assert_parses(r"\underline{x}");
}

// =============================================================================
// Rule Parser Tests
// =============================================================================

#[test]
fn rule_parser_should_not_fail() {
    assert_parses(r"\rule{1em}{2em}");
    assert_parses(r"\rule[0.5em]{1em}{2em}");
}

// =============================================================================
// Kern Parser Tests
// =============================================================================

#[test]
fn kern_parser_should_not_fail() {
    assert_parses(r"x\kern{1em}y");
    assert_parses(r"x\kern1em y");
}

// =============================================================================
// Lap Parser Tests
// =============================================================================

#[test]
fn lap_parser_should_not_fail() {
    assert_parses(r"\llap{x}");
    assert_parses(r"\rlap{x}");
    assert_parses(r"\clap{x}");
}

// =============================================================================
// Style Change Parser Tests
// =============================================================================

#[test]
fn style_change_parser_should_not_fail() {
    assert_parses(r"\displaystyle x");
    assert_parses(r"\textstyle x");
    assert_parses(r"\scriptstyle x");
    assert_parses(r"\scriptscriptstyle x");
}

// =============================================================================
// Font Parser Tests
// =============================================================================

#[test]
fn font_parser_should_not_fail() {
    assert_parses(r"\mathbb{R}");
    assert_parses(r"\mathcal{L}");
    assert_parses(r"\mathbf{x}");
    assert_parses(r"\mathit{x}");
    assert_parses(r"\mathscr{F}");
    assert_parses(r"\mathsf{x}");
    assert_parses(r"\mathtt{x}");
    assert_parses(r"\mathfrak{g}");
    assert_parses(r"\mathrm{d}");
}

// =============================================================================
// Raise Parser Tests
// =============================================================================

#[test]
fn raise_parser_should_not_fail() {
    assert_parses(r"\raisebox{0.25em}{x}");
    assert_parses(r"\raisebox{-0.25em}{x}");
}

// =============================================================================
// Comment Parser Tests
// =============================================================================

#[test]
fn comment_parser_should_not_fail() {
    assert_parses("x % comment\ny");
    assert_parses(r"\frac{1}{2} % numerator is 1");
}

// =============================================================================
// Environment Parser Tests
// =============================================================================

#[test]
fn matrix_environment_should_not_fail() {
    assert_parses(r"\begin{matrix} a & b \\ c & d \end{matrix}");
    assert_parses(r"\begin{pmatrix} 1 & 2 \\ 3 & 4 \end{pmatrix}");
    assert_parses(r"\begin{bmatrix} 1 & 2 \\ 3 & 4 \end{bmatrix}");
    assert_parses(r"\begin{vmatrix} 1 & 2 \\ 3 & 4 \end{vmatrix}");
    assert_parses(r"\begin{Vmatrix} 1 & 2 \\ 3 & 4 \end{Vmatrix}");
}

#[test]
fn array_environment_should_not_fail() {
    assert_parses(r"\begin{array}{cc} a & b \\ c & d \end{array}");
    assert_parses(r"\begin{array}{|c|c|} a & b \\ \hline c & d \end{array}");
}

#[test]
fn cases_environment_should_not_fail() {
    assert_parses(r"\begin{cases} x & \text{if } x > 0 \\ -x & \text{otherwise} \end{cases}");
}

#[test]
fn aligned_environment_should_not_fail() {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;
    let result = parse_tree(r"\begin{aligned} a &= b \\ c &= d \end{aligned}", conf);
    assert!(result.is_ok());
}

#[test]
fn align_environment_should_not_fail() {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;
    let result = parse_tree(r"\begin{align} a &= b \\ c &= d \end{align}", conf);
    assert!(result.is_ok());
}

// =============================================================================
// Accent Parser Tests
// =============================================================================

#[test]
fn accent_parser_should_not_fail() {
    assert_parses(r"\acute{x}");
    assert_parses(r"\grave{x}");
    assert_parses(r"\ddot{x}");
    assert_parses(r"\tilde{x}");
    assert_parses(r"\bar{x}");
    assert_parses(r"\breve{x}");
    assert_parses(r"\check{x}");
    assert_parses(r"\hat{x}");
    assert_parses(r"\vec{x}");
    assert_parses(r"\dot{x}");
}

#[test]
fn wide_accents_should_not_fail() {
    assert_parses(r"\widehat{xyz}");
    assert_parses(r"\widetilde{xyz}");
    assert_parses(r"\overrightarrow{AB}");
    assert_parses(r"\overleftarrow{AB}");
}

// =============================================================================
// Under-accent Parser Tests
// =============================================================================

#[test]
fn underaccent_parser_should_not_fail() {
    assert_parses(r"\underbrace{x+y}");
    assert_parses(r"\underbar{x}");
}

// =============================================================================
// Extensible Arrow Parser Tests
// =============================================================================

#[test]
fn extensible_arrow_parser_should_not_fail() {
    assert_parses(r"\xrightarrow{text}");
    assert_parses(r"\xleftarrow{text}");
    assert_parses(r"\xrightarrow[below]{above}");
}

// =============================================================================
// Horizontal Brace Parser Tests
// =============================================================================

#[test]
fn horizontal_brace_parser_should_not_fail() {
    assert_parses(r"\overbrace{x+y}");
    assert_parses(r"\overbrace{x+y}^{text}");
    assert_parses(r"\underbrace{x+y}_{text}");
}

// =============================================================================
// Box Parser Tests
// =============================================================================

#[test]
fn boxed_parser_should_not_fail() {
    assert_parses(r"\boxed{x=2}");
}

#[test]
fn fbox_parser_should_not_fail() {
    assert_parses(r"\fbox{content}");
}

// =============================================================================
// Strike-through Parser Tests
// =============================================================================

#[test]
fn cancel_parser_should_not_fail() {
    assert_parses(r"\cancel{x}");
    assert_parses(r"\bcancel{x}");
    assert_parses(r"\xcancel{x}");
    assert_parses(r"\cancelto{0}{x}");
}

// =============================================================================
// Phantom Parser Tests
// =============================================================================

#[test]
fn phantom_parser_should_not_fail() {
    assert_parses(r"\phantom{x}");
    assert_parses(r"\vphantom{x}");
    assert_parses(r"\hphantom{x}");
}

// =============================================================================
// Smash Parser Tests
// =============================================================================

#[test]
fn smash_parser_should_not_fail() {
    assert_parses(r"\smash{x}");
    assert_parses(r"\smash[b]{x}");
    assert_parses(r"\smash[t]{x}");
}

// =============================================================================
// Verb Parser Tests
// =============================================================================

#[test]
fn verb_parser_should_not_fail() {
    assert_parses(r"\verb|x^2|");
    assert_parses(r"\verb!x_3!");
}
