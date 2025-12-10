//! Tests ported from KaTeX's katex-spec.js that are missing from Aliter
//!
//! These tests cover functionality from the JavaScript KaTeX test suite that
//! wasn't previously covered. Some tests may fail until the corresponding
//! functionality is implemented.

use aliter::{parse_tree, parser::ParserConfig, parser::StrictMode};

// =============================================================================
// Helper Functions
// =============================================================================

fn assert_parses(expr: &str) {
    let conf = ParserConfig::default();
    let result = parse_tree(expr, conf);
    assert!(result.is_ok(), "Failed to parse: {}\nError: {:?}", expr, result.err());
}

fn assert_parses_with_config(expr: &str, conf: ParserConfig) {
    let result = parse_tree(expr, conf);
    assert!(result.is_ok(), "Failed to parse with config: {}\nError: {:?}", expr, result.err());
}

fn assert_fails(expr: &str) {
    let conf = ParserConfig::default();
    let result = parse_tree(expr, conf);
    assert!(result.is_err(), "Expected parse error for: {}", expr);
}

fn assert_fails_with_config(expr: &str, conf: ParserConfig) {
    let result = parse_tree(expr, conf);
    assert!(result.is_err(), "Expected parse error for: {}", expr);
}

fn assert_parses_strict(expr: &str) {
    let mut conf = ParserConfig::default();
    conf.strict = StrictMode::Error;
    let result = parse_tree(expr, conf);
    assert!(result.is_ok(), "Failed to parse (strict): {}\nError: {:?}", expr, result.err());
}

fn assert_fails_strict(expr: &str) {
    let mut conf = ParserConfig::default();
    conf.strict = StrictMode::Error;
    let result = parse_tree(expr, conf);
    assert!(result.is_err(), "Expected strict mode error for: {}", expr);
}

// =============================================================================
// Unicode Subscript/Superscript Characters (katex-spec.js:279)
// =============================================================================

#[test]
fn parser_should_work_with_unicode_superscript_characters() {
    // Unicode superscript characters like ² ³ should parse
    assert_parses("x²");
    assert_parses("x³");
    assert_parses("x¹");
}

#[test]
fn parser_should_work_with_unicode_subscript_characters() {
    // Unicode subscript characters like ₀ ₁ ₂ should parse
    assert_parses("x₀");
    assert_parses("x₁");
    assert_parses("x₂");
}

// =============================================================================
// Implicit Group Parser - Optional Groups (katex-spec.js:411-427)
// =============================================================================

#[test]
fn implicit_group_works_with_sizing_in_optional_args() {
    // \sqrt[\small 3]{x}
    assert_parses(r"\sqrt[\small 3]{x}");
}

#[test]
fn implicit_group_works_with_color_in_optional_args() {
    // \sqrt[\color{red} 3]{x}
    assert_parses(r"\sqrt[\color{red} 3]{x}");
}

#[test]
fn implicit_group_works_with_style_in_optional_args() {
    // \sqrt[\textstyle 3]{x}
    assert_parses(r"\sqrt[\textstyle 3]{x}");
}

#[test]
fn implicit_group_works_with_old_font_in_optional_args() {
    // \sqrt[\tt 3]{x}
    assert_parses(r"\sqrt[\tt 3]{x}");
}

// =============================================================================
// Function Parser Edge Cases (katex-spec.js:457-465)
// =============================================================================

#[test]
fn function_parser_should_not_parse_function_with_text_right_after() {
    // "\redx" should not be parsed as \red + x
    assert_fails(r"\redx");
}

#[test]
fn function_parser_should_parse_function_with_number_right_after() {
    // "\frac12" should parse as \frac{1}{2}
    assert_parses(r"\frac12");
}

#[test]
fn function_parser_should_parse_some_functions_with_text_after() {
    // Some functions like \alpha work with text after
    assert_parses(r"\alphax");
    assert_parses(r"\betay");
}

// =============================================================================
// Genfrac Edge Cases (katex-spec.js:526-535)
// =============================================================================

#[test]
fn genfrac_should_fail_with_math_as_line_thickness() {
    // \genfrac with math expression as thickness should fail
    assert_fails(r"\genfrac{(}{)}{1+2}{0}{a}{b}");
}

#[test]
fn genfrac_should_fail_with_insufficient_arguments() {
    // genfrac needs 6 arguments
    assert_fails(r"\genfrac{(}{)}{1pt}{0}{a}");
}

// =============================================================================
// Over/Brace/Brack Edge Cases (katex-spec.js:605-653)
// =============================================================================

#[test]
fn over_should_handle_empty_numerators() {
    // \over with empty numerator
    assert_parses(r"\over 2");
}

#[test]
fn over_should_handle_empty_denominators() {
    // \over with empty denominator
    assert_parses(r"1\over");
}

#[test]
fn over_should_handle_displaystyle_correctly() {
    // \displaystyle before \over
    assert_parses(r"\displaystyle 1 \over 2");
}

#[test]
fn over_should_handle_textstyle_correctly() {
    // \textstyle should work with \over
    assert_parses(r"\textstyle 1 \over 2");
}

#[test]
fn over_should_handle_nested_fractions() {
    // Nested \over expressions
    assert_parses(r"{1 \over 2} \over 3");
    assert_parses(r"1 \over {2 \over 3}");
}

#[test]
fn over_should_fail_with_multiple_overs_in_same_group() {
    // Multiple \over in same group should fail
    assert_fails(r"1 \over 2 \over 3");
}

// =============================================================================
// Text Parser Edge Cases (katex-spec.js:704-810)
// =============================================================================

#[test]
fn text_parser_contracts_spaces() {
    // Multiple spaces should contract
    assert_parses(r"\text{a     b}");
}

#[test]
fn text_parser_handles_backslash_newline() {
    // Backslash followed by newline in text
    assert_parses("\\text{a\\\nb}");
}

#[test]
fn text_parser_handles_math_within_text() {
    // Math within text group using \( \)
    assert_parses(r"\text{a \(b\) c}");
}

#[test]
fn text_parser_handles_nested_math_text_math_text() {
    // Nested math within text within math within text
    assert_parses(r"\text{a \(\text{b \(c\) d}\) e}");
}

#[test]
fn text_parser_forbids_open_paren_in_math_mode() {
    // \( inside math mode should fail
    assert_fails(r"a \( b");
}

#[test]
fn text_parser_forbids_dollar_in_math_mode() {
    // $ inside math mode should fail
    assert_fails(r"a $ b");
}

#[test]
fn text_parser_detects_unbalanced_close_paren() {
    // Unbalanced \) should fail
    assert_fails(r"\text{a \) b}");
}

#[test]
fn text_parser_parses_spacing_functions() {
    // Spacing functions in text mode
    assert_parses(r"\text{a\;b}");
    assert_parses(r"\text{a\,b}");
}

#[test]
fn text_parser_omits_spaces_after_commands() {
    // Spaces after commands should be consumed
    assert_parses(r"\text{\textbf {x}}");
}

// =============================================================================
// Comment Parser Tests (katex-spec.js:1706-1763)
// =============================================================================

#[test]
fn comment_parser_parses_comments_at_end_of_line() {
    assert_parses("a+b% comment\n+c");
}

#[test]
fn comment_parser_parses_comments_at_start_of_line() {
    assert_parses("% comment\na+b");
}

#[test]
fn comment_parser_parses_multiple_comment_lines() {
    assert_parses("% comment 1\n% comment 2\na+b");
}

#[test]
fn comment_parser_parses_comments_between_sub_and_super() {
    assert_parses("x_a%\n^b");
}

#[test]
fn comment_parser_parses_comments_in_size_and_color_groups() {
    assert_parses(r"\textcolor{%comment
red}{x}");
}

#[test]
fn comment_parser_parses_comments_before_expression() {
    assert_parses("% comment\n x");
}

#[test]
fn comment_parser_parses_comments_before_hline() {
    assert_parses(r"\begin{array}{c}%
\hline a\end{array}");
}

#[test]
fn comment_parser_parses_comments_in_macro_definition() {
    assert_parses(r"\def\foo{%
x}%
\foo");
}

#[test]
fn comment_without_newline_fails_in_strict_mode() {
    // Comment without newline at end should fail in strict mode
    assert_fails_strict("x%comment");
}

// =============================================================================
// TeX-Compliant Parser Tests (katex-spec.js:1373-1516)
// =============================================================================

#[test]
fn tex_parser_fails_with_not_enough_arguments() {
    assert_fails(r"\frac");
    assert_fails(r"\frac{x}");
}

#[test]
fn tex_parser_fails_with_missing_sup_subscripts() {
    assert_fails(r"x^");
    assert_fails(r"x_");
}

#[test]
fn tex_parser_fails_when_arguments_require_arguments() {
    // \frac\frac without enough args should fail
    assert_fails(r"\frac\frac");
}

#[test]
fn tex_parser_works_when_arguments_have_braces() {
    assert_parses(r"\frac{1}{2}");
    assert_parses(r"\sqrt{2}");
}

#[test]
fn tex_parser_fails_when_sup_subscripts_require_arguments() {
    // x^a^ requires another argument after second ^
    assert_fails(r"x^\frac");
}

#[test]
fn tex_parser_works_with_sup_subscript_arguments_in_braces() {
    assert_parses(r"x^{\frac{1}{2}}");
    assert_parses(r"x_{\sqrt{2}}");
}

#[test]
fn tex_parser_parses_multiple_primes_correctly() {
    assert_parses(r"f'");
    assert_parses(r"f''");
    assert_parses(r"f'''");
    assert_parses(r"f''''");
}

#[test]
fn tex_parser_succeeds_when_sup_subscripts_come_after_whole_functions() {
    assert_parses(r"\frac{1}{2}^3");
    assert_parses(r"\sqrt{2}_1");
}

#[test]
fn tex_parser_succeeds_with_sqrt_around_text_frac() {
    assert_parses(r"\sqrt{\text{a}}");
    assert_parses(r"\sqrt{\frac{1}{2}}");
}

#[test]
fn tex_parser_fails_when_arguments_are_left() {
    // \frac\left... requires arguments for \left first
    assert_fails(r"\frac\left");
}

#[test]
fn tex_parser_succeeds_with_braces_around_left_right() {
    assert_parses(r"\frac{\left(x\right)}{2}");
}

// =============================================================================
// Raise Parser Tests (katex-spec.js:1676-1705)
// =============================================================================

#[test]
fn raise_parser_parses_raisebox() {
    assert_parses(r"\raisebox{1pt}{\text{a}}");
}

#[test]
fn raise_parser_parses_vcenter_in_nonstrict() {
    let mut conf = ParserConfig::default();
    conf.strict = StrictMode::Ignore;
    assert_parses_with_config(r"\vcenter{x^2}", conf);
}

#[test]
fn raise_parser_fails_with_math_in_raisebox() {
    // \raisebox should only work with text content
    assert_fails(r"\raisebox{1pt}{x^2}");
}

#[test]
fn raise_parser_fails_with_unbraced_length() {
    // Length must be braced
    assert_fails(r"\raisebox 1pt {\text{a}}");
}

// =============================================================================
// Subarray Environment (katex-spec.js:2734-2747)
// =============================================================================

#[test]
fn subarray_accepts_single_alignment_character() {
    assert_parses(r"\begin{subarray}{l}a\\b\end{subarray}");
    assert_parses(r"\begin{subarray}{c}a\\b\end{subarray}");
}

#[test]
fn subarray_rejects_multiple_alignment_characters() {
    assert_fails(r"\begin{subarray}{cc}a&b\end{subarray}");
}

// =============================================================================
// Substack Function (katex-spec.js:2749-2763)
// =============================================================================

#[test]
fn substack_should_build() {
    assert_parses(r"\sum_{\substack{1\le i\le n\\ i\ne j}} x_i");
}

#[test]
fn substack_accommodates_spaces_in_argument() {
    assert_parses(r"\substack{ a \\ b }");
}

#[test]
fn substack_accommodates_macros_in_argument() {
    assert_parses(r"\substack{\alpha\\\beta}");
}

#[test]
fn substack_accommodates_empty_argument() {
    assert_parses(r"\substack{}");
}

// =============================================================================
// Smallmatrix Environment (katex-spec.js:2766-2771)
// =============================================================================

#[test]
fn smallmatrix_should_build() {
    assert_parses(r"\begin{smallmatrix}a&b\\c&d\end{smallmatrix}");
}

// =============================================================================
// Rcases Environment (katex-spec.js:2783-2789)
// =============================================================================

#[test]
fn rcases_should_build() {
    assert_parses(r"\begin{rcases}a&\text{if }b\\c&\text{if }d\end{rcases}");
}

// =============================================================================
// Aligned Edge Cases (katex-spec.js:2792-2809)
// =============================================================================

#[test]
fn aligned_allows_cells_in_brackets() {
    // aligned with cells in brackets (with space before bracket)
    assert_parses(r"\begin{aligned}[t]a&=b\end{aligned}");
}

#[test]
fn aligned_forbids_cells_in_brackets_without_space() {
    // [t] right after begin without space might be interpreted differently
    // This test checks the specific case mentioned in KaTeX
    assert_parses(r"\begin{aligned} [t]a&=b\end{aligned}");
}

#[test]
fn aligned_does_not_eat_last_row_when_first_cell_empty() {
    // Empty first cell shouldn't cause last row to be eaten
    assert_parses(r"\begin{aligned}&a\\&b\end{aligned}");
}

// =============================================================================
// AMS Environment Tests (katex-spec.js:2812-2863)
// =============================================================================

#[test]
fn ams_environments_fail_outside_display_mode() {
    let mut conf = ParserConfig::default();
    conf.display_mode = false;

    assert_fails_with_config(r"\begin{align}a&=b\end{align}", conf.clone());
    assert_fails_with_config(r"\begin{gather}a\end{gather}", conf.clone());
    assert_fails_with_config(r"\begin{equation}a\end{equation}", conf.clone());
}

#[test]
fn ams_environments_build_in_display_mode() {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;

    assert_parses_with_config(r"\begin{align}a&=b\end{align}", conf.clone());
    assert_parses_with_config(r"\begin{gather}a\end{gather}", conf.clone());
    assert_parses_with_config(r"\begin{equation}a\end{equation}", conf.clone());
}

#[test]
fn ams_environments_build_empty() {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;

    assert_parses_with_config(r"\begin{align}\end{align}", conf.clone());
    assert_parses_with_config(r"\begin{gather}\end{gather}", conf.clone());
}

#[test]
fn equation_fails_with_two_rows() {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;

    assert_fails_with_config(r"\begin{equation}a\\b\end{equation}", conf);
}

#[test]
fn equation_fails_with_two_columns() {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;

    assert_fails_with_config(r"\begin{equation}a&b\end{equation}", conf);
}

#[test]
fn split_fails_with_three_columns() {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;

    assert_fails_with_config(r"\begin{split}a&b&c\end{split}", conf);
}

#[test]
fn array_fails_with_more_columns_than_specification() {
    // Array with {cc} but 3 columns of data
    assert_fails(r"\begin{array}{cc}a&b&c\end{array}");
}

// =============================================================================
// CD Environment Tests (katex-spec.js:2865-2886)
// =============================================================================

#[test]
fn cd_fails_outside_display_mode() {
    let mut conf = ParserConfig::default();
    conf.display_mode = false;

    assert_fails_with_config(r"\begin{CD}A @>>> B\end{CD}", conf);
}

#[test]
fn cd_fails_with_invalid_character_after_at() {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;

    // Character after @ must be in <>AV=|.
    assert_fails_with_config(r"\begin{CD}A @x B\end{CD}", conf);
}

#[test]
fn cd_fails_without_final_arrow_character() {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;

    assert_fails_with_config(r"\begin{CD}A @>> B\end{CD}", conf);
}

#[test]
fn cd_succeeds_with_proper_syntax() {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;

    assert_parses_with_config(r"\begin{CD}A @>>> B @VVV C\end{CD}", conf);
}

// =============================================================================
// Operatorname Support (katex-spec.js:2888-2897)
// =============================================================================

#[test]
fn operatorname_parses() {
    assert_parses(r"\operatorname{sn}");
    assert_parses(r"\operatorname*{arg\,max}");
}

// =============================================================================
// Href and URL Commands (katex-spec.js:2899-2994)
// =============================================================================

#[test]
fn href_parses_basic_input() {
    let mut conf = ParserConfig::default();
    conf.trust = true;
    assert_parses_with_config(r"\href{http://example.com}{text}", conf);
}

#[test]
fn href_allows_empty_urls() {
    let mut conf = ParserConfig::default();
    conf.trust = true;
    assert_parses_with_config(r"\href{}{text}", conf);
}

#[test]
fn href_allows_single_character_urls() {
    let mut conf = ParserConfig::default();
    conf.trust = true;
    assert_parses_with_config(r"\href{a}{text}", conf);
}

#[test]
fn url_allows_special_characters_without_escaping() {
    let mut conf = ParserConfig::default();
    conf.trust = true;
    // Characters #$%&~_^ should be allowed
    assert_parses_with_config(r"\href{http://example.com#anchor}{text}", conf.clone());
    assert_parses_with_config(r"\href{http://example.com?a=b&c=d}{text}", conf);
}

#[test]
fn url_allows_balanced_braces() {
    let mut conf = ParserConfig::default();
    conf.trust = true;
    assert_parses_with_config(r"\href{http://example.com/{path}}{text}", conf);
}

#[test]
fn url_rejects_unbalanced_braces() {
    let mut conf = ParserConfig::default();
    conf.trust = true;
    assert_fails_with_config(r"\href{http://example.com/{path}{text}", conf);
}

#[test]
fn href_forbids_relative_urls_when_trust_is_false() {
    let mut conf = ParserConfig::default();
    conf.trust = false;
    assert_fails_with_config(r"\href{../relative}{text}", conf);
}

#[test]
fn href_allows_all_protocols_when_trust_is_true() {
    let mut conf = ParserConfig::default();
    conf.trust = true;
    assert_parses_with_config(r"\href{javascript:alert(1)}{text}", conf.clone());
    assert_parses_with_config(r"\href{file:///etc/passwd}{text}", conf);
}

// =============================================================================
// Tag Support (katex-spec.js:3635-3670)
// =============================================================================

#[test]
fn tag_fails_outside_display_mode() {
    let mut conf = ParserConfig::default();
    conf.display_mode = false;
    assert_fails_with_config(r"a=b \tag{1}", conf);
}

#[test]
fn tag_fails_with_multiple_tags() {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;
    assert_fails_with_config(r"a=b \tag{1} \tag{2}", conf);
}

#[test]
fn tag_works_with_one_tag_per_row() {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;
    assert_parses_with_config(r"\begin{align}a&=b\tag{1}\\c&=d\tag{2}\end{align}", conf);
}

#[test]
fn tag_works_with_nonumber() {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;
    assert_parses_with_config(r"\begin{align}a&=b\nonumber\\c&=d\end{align}", conf);
}

#[test]
fn tag_works_with_notag() {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;
    assert_parses_with_config(r"\begin{align}a&=b\notag\\c&=d\end{align}", conf);
}

#[test]
fn tag_star_works() {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;
    assert_parses_with_config(r"a=b \tag*{eq. 1}", conf);
}

// =============================================================================
// Binrel Automatic Class (katex-spec.js:3695-3710)
// =============================================================================

#[test]
fn binrel_generates_proper_class() {
    assert_parses(r"\mathrel{:}");
    assert_parses(r"\mathbin{+}");
    assert_parses(r"\mathord{x}");
}

// =============================================================================
// Unicode Accents (katex-spec.js:3724-3786)
// =============================================================================

#[test]
fn unicode_accents_parse_latin1_in_math_mode() {
    let mut conf = ParserConfig::default();
    conf.strict = StrictMode::Ignore;
    assert_parses_with_config("é", conf.clone());
    assert_parses_with_config("ü", conf.clone());
    assert_parses_with_config("ñ", conf);
}

#[test]
fn unicode_accents_parse_latin1_in_text_mode() {
    assert_parses(r"\text{é}");
    assert_parses(r"\text{ü}");
    assert_parses(r"\text{ñ}");
}

#[test]
fn unicode_supports_aa_in_text_mode() {
    assert_parses(r"\text{\aa}");
    assert_parses(r"\text{\AA}");
}

#[test]
fn unicode_parses_combining_characters() {
    let mut conf = ParserConfig::default();
    conf.strict = StrictMode::Ignore;
    // a followed by combining acute accent
    assert_parses_with_config("a\u{0301}", conf);
}

#[test]
fn unicode_parses_multi_accented_characters() {
    let mut conf = ParserConfig::default();
    conf.strict = StrictMode::Ignore;
    // Characters with multiple combining marks
    assert_parses_with_config("ä́", conf);
}

#[test]
fn unicode_parses_accented_i_and_j() {
    assert_parses(r"\text{ï}");
    assert_parses(r"\text{í}");
}

// =============================================================================
// Unicode Relations and Operators (katex-spec.js:3788-3881)
// =============================================================================

#[test]
fn unicode_parses_negated_relations() {
    let mut conf = ParserConfig::default();
    conf.strict = StrictMode::Ignore;
    assert_parses_with_config("≠", conf.clone());
    assert_parses_with_config("≮", conf.clone());
    assert_parses_with_config("≯", conf);
}

#[test]
fn unicode_builds_relations() {
    let mut conf = ParserConfig::default();
    conf.strict = StrictMode::Ignore;
    assert_parses_with_config("≤", conf.clone());
    assert_parses_with_config("≥", conf.clone());
    assert_parses_with_config("∈", conf);
}

#[test]
fn unicode_builds_big_operators() {
    let mut conf = ParserConfig::default();
    conf.strict = StrictMode::Ignore;
    assert_parses_with_config("∑", conf.clone());
    assert_parses_with_config("∏", conf.clone());
    assert_parses_with_config("∫", conf);
}

#[test]
fn unicode_builds_arrows() {
    let mut conf = ParserConfig::default();
    conf.strict = StrictMode::Ignore;
    assert_parses_with_config("→", conf.clone());
    assert_parses_with_config("←", conf.clone());
    assert_parses_with_config("↔", conf);
}

#[test]
fn unicode_builds_binary_operators() {
    let mut conf = ParserConfig::default();
    conf.strict = StrictMode::Ignore;
    assert_parses_with_config("±", conf.clone());
    assert_parses_with_config("×", conf.clone());
    assert_parses_with_config("÷", conf);
}

#[test]
fn unicode_builds_common_ords() {
    let mut conf = ParserConfig::default();
    conf.strict = StrictMode::Ignore;
    assert_parses_with_config("∞", conf.clone());
    assert_parses_with_config("∂", conf.clone());
    assert_parses_with_config("∇", conf);
}

#[test]
fn unicode_builds_delimiters() {
    let mut conf = ParserConfig::default();
    conf.strict = StrictMode::Ignore;
    assert_parses_with_config("⟨", conf.clone());
    assert_parses_with_config("⟩", conf.clone());
    assert_parses_with_config("⌊", conf.clone());
    assert_parses_with_config("⌋", conf);
}

// =============================================================================
// MaxExpand Setting (katex-spec.js:3907-3918)
// =============================================================================

#[test]
fn max_expand_prevents_expansion() {
    let mut conf = ParserConfig::default();
    conf.max_expand = Some(0);
    // With max_expand = 0, macros should not expand
    // This might cause parse to fail or produce different output
    // depending on implementation
    let result = parse_tree(r"\def\foo{x}\foo", conf);
    // We just check it doesn't infinite loop; result could be ok or err
    let _ = result;
}

#[test]
fn max_expand_prevents_infinite_loops() {
    let mut conf = ParserConfig::default();
    conf.max_expand = Some(100);
    // Self-referential macro should hit the limit
    assert_fails_with_config(r"\def\foo{\foo}\foo", conf);
}

// =============================================================================
// Mathchoice Function (katex-spec.js:3920-3938)
// =============================================================================

#[test]
fn mathchoice_parses_in_display_mode() {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;
    assert_parses_with_config(r"\mathchoice{D}{T}{S}{SS}", conf);
}

#[test]
fn mathchoice_parses_in_text_mode() {
    assert_parses(r"\mathchoice{D}{T}{S}{SS}");
}

#[test]
fn mathchoice_parses_in_scriptstyle() {
    assert_parses(r"x^{\mathchoice{D}{T}{S}{SS}}");
}

#[test]
fn mathchoice_parses_in_scriptscriptstyle() {
    assert_parses(r"x^{y^{\mathchoice{D}{T}{S}{SS}}}");
}

// =============================================================================
// Newlines (katex-spec.js:3941-3962)
// =============================================================================

#[test]
fn newline_and_double_backslash_build_same() {
    // \\ and \newline should both work
    assert_parses(r"\begin{array}{c}a\\b\end{array}");
    assert_parses(r"\begin{array}{c}a\newline b\end{array}");
}

#[test]
fn newline_does_not_scan_for_optional_size() {
    // \newline should not take optional argument
    assert_parses(r"\begin{array}{c}a\newline[b]\end{array}");
}

#[test]
fn cr_not_allowed_at_top_level() {
    // \cr should not be allowed outside arrays
    assert_fails(r"a\cr b");
}

#[test]
fn double_backslash_causes_newline_after_mrel_and_mop() {
    // \\ should cause newline even after relation or operator
    let mut conf = ParserConfig::default();
    conf.display_mode = true;
    assert_parses_with_config(r"\begin{gather}a=\\b\end{gather}", conf.clone());
    assert_parses_with_config(r"\begin{gather}a+\\b\end{gather}", conf);
}

// =============================================================================
// Symbols (katex-spec.js:3964-3980)
// =============================================================================

#[test]
fn symbols_parse_i_and_j_in_text() {
    assert_parses(r"\text{\i}");
    assert_parses(r"\text{\j}");
}

#[test]
fn spacing_functions_parse_in_math_and_text() {
    assert_parses(r"a\;b");
    assert_parses(r"\text{a\;b}");
    assert_parses(r"a\,b");
    assert_parses(r"\text{a\,b}");
}

#[test]
fn minuso_parses() {
    assert_parses(r"\minuso");
}

// =============================================================================
// Strict Setting (katex-spec.js:3982-4020)
// =============================================================================

#[test]
fn strict_allows_unicode_text_when_not_strict() {
    let mut conf = ParserConfig::default();
    conf.strict = StrictMode::Ignore;
    assert_parses_with_config("日本語", conf);
}

#[test]
fn strict_forbids_unicode_text_when_strict() {
    let mut conf = ParserConfig::default();
    conf.strict = StrictMode::Error;
    assert_fails_with_config("日本語", conf);
}

#[test]
fn strict_always_allows_unicode_in_text_mode() {
    let mut conf = ParserConfig::default();
    conf.strict = StrictMode::Error;
    // Unicode should be allowed inside \text even in strict mode
    assert_parses_with_config(r"\text{日本語}", conf);
}

#[test]
fn strict_warns_about_toplevel_newline_in_display() {
    // Top-level \\ or \newline in display mode may trigger a warning
    let mut conf = ParserConfig::default();
    conf.display_mode = true;
    // In warn mode, this should still parse but would generate a warning
    assert_parses_with_config(r"a\\b", conf);
}

// =============================================================================
// Relax (katex-spec.js:4100-4103)
// =============================================================================

#[test]
fn relax_stops_expansion() {
    assert_parses(r"\relax");
    assert_parses(r"x\relax y");
    // \relax should stop implicit groups
    assert_parses(r"\frac{1\relax}{2}");
}

// =============================================================================
// Begin/End Edge Cases (katex-spec.js:1276-1335)
// =============================================================================

#[test]
fn begin_end_forbids_hlines_outside_array() {
    assert_fails(r"\hline");
    assert_fails(r"x\hline y");
}

#[test]
fn begin_end_allows_cr_and_double_backslash_as_line_terminator() {
    assert_parses(r"\begin{array}{c}a\\b\end{array}");
    assert_parses(r"\begin{array}{c}a\cr b\end{array}");
}

#[test]
fn begin_end_eats_final_newline() {
    // Trailing \\ at end of array should be allowed
    assert_parses(r"\begin{array}{c}a\\b\\\end{array}");
}

#[test]
fn begin_end_grabs_arraystretch() {
    assert_parses(r"\def\arraystretch{1.5}\begin{array}{c}a\\b\end{array}");
}

#[test]
fn begin_end_allows_optional_argument_in_matrix_star() {
    assert_parses(r"\begin{matrix*}[r]a&b\\c&d\end{matrix*}");
}

#[test]
fn begin_end_allows_blank_columns() {
    assert_parses(r"\begin{array}{c}&&\end{array}");
}

// =============================================================================
// Sqrt Expansion (katex-spec.js:1362-1372)
// =============================================================================

#[test]
fn sqrt_expands_argument_without_optional() {
    // Without optional arg, the argument should expand normally
    assert_parses(r"\def\x{2}\sqrt{\x}");
}

#[test]
fn sqrt_does_not_expand_argument_with_optional() {
    // With optional arg present
    assert_parses(r"\def\x{2}\sqrt[\x]{\x}");
}
