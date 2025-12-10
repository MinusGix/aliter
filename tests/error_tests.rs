use aliter::{parse_tree, parser::ParserConfig, parser::ParseError};

// Helper to check that parsing fails with expected error
fn assert_parse_error(expr: &str, expected_error: ParseError) {
    let conf = ParserConfig::default();
    let result = parse_tree(expr, conf);
    assert!(result.is_err(), "Expected parse error for: {}", expr);

    // For now, just check that we get an error
    // TODO: Match specific error types more precisely
    if let Err(err) = result {
        // Convert both to debug strings for comparison
        let err_debug = format!("{:?}", err);
        let expected_debug = format!("{:?}", expected_error);

        // Extract just the variant name (before any parentheses)
        let err_variant = err_debug.split('(').next().unwrap_or("");
        let expected_variant = expected_debug.split('(').next().unwrap_or("");

        assert_eq!(
            err_variant, expected_variant,
            "Expected error {:?} but got {:?} for expression: {}",
            expected_error, err, expr
        );
    }
}

// Simpler helper that just checks for any error
fn assert_fails(expr: &str) {
    let conf = ParserConfig::default();
    let result = parse_tree(expr, conf);
    assert!(result.is_err(), "Expected parse error for: {}", expr);
}

// =============================================================================
// Parser Error Tests (from KaTeX errors-spec.js)
// =============================================================================

#[test]
fn rejects_repeated_infix_operators() {
    // 1\over 2\over 3
    assert_parse_error(r"1\over 2\over 3", ParseError::OnlyOneInfixOperatorPerGroup);
}

#[test]
fn rejects_conflicting_infix_operators() {
    // 1\over 2\choose 3
    assert_parse_error(r"1\over 2\choose 3", ParseError::OnlyOneInfixOperatorPerGroup);
}

#[test]
fn rejects_superscript_at_end_of_group() {
    // {1^}
    assert_parse_error(r"{1^}", ParseError::ExpectedGroup);
}

#[test]
fn rejects_subscript_at_end_of_input() {
    // 1_
    assert_parse_error(r"1_", ParseError::ExpectedGroup);
}

#[test]
fn rejects_sqrt_as_argument_to_superscript() {
    // 1^\sqrt{2}
    assert_parse_error(r"1^\sqrt{2}", ParseError::FunctionNoArguments);
}

#[test]
fn rejects_limits_without_operator() {
    // \alpha\limits\omega
    assert_parse_error(r"\alpha\limits\omega", ParseError::ExpectedLimitControls);
}

#[test]
fn rejects_limits_at_beginning() {
    // \limits\omega
    assert_parse_error(r"\limits\omega", ParseError::ExpectedLimitControls);
}

#[test]
fn rejects_double_superscripts() {
    // 1^2^3
    assert_parse_error(r"1^2^3", ParseError::DoubleSuperscript);

    // 1^{2+3}_4^5
    assert_parse_error(r"1^{2+3}_4^5", ParseError::DoubleSuperscript);
}

#[test]
fn rejects_double_superscripts_with_primes() {
    // 1'_2^3
    assert_parse_error(r"1'_2^3", ParseError::DoubleSuperscript);

    // 1^2'
    assert_parse_error(r"1^2'", ParseError::DoubleSuperscript);

    // 1^2_3'
    assert_parse_error(r"1^2_3'", ParseError::DoubleSuperscript);

    // 1'_2'
    assert_parse_error(r"1'_2'", ParseError::DoubleSuperscript);
}

#[test]
fn rejects_double_subscripts() {
    // 1_2_3
    assert_parse_error(r"1_2_3", ParseError::DoubleSubscript);

    // 1_{2+3}^4_5
    assert_parse_error(r"1_{2+3}^4_5", ParseError::DoubleSubscript);
}

#[test]
fn reports_unknown_environments() {
    // \begin{foo}bar\end{foo}
    assert_fails(r"\begin{foo}bar\end{foo}");
}

#[test]
fn reports_mismatched_environments() {
    // \begin{pmatrix}1&2\\3&4\end{bmatrix}+5
    assert_fails(r"\begin{pmatrix}1&2\\3&4\end{bmatrix}+5");
}

#[test]
fn rejects_math_mode_functions_in_text_mode() {
    // \text{\sqrt2 is irrational}
    assert_parse_error(
        r"\text{\sqrt2 is irrational}",
        ParseError::FunctionUnusableTextMode
    );
}

#[test]
fn rejects_text_mode_only_functions_in_math_mode() {
    // $ is text-mode only
    assert_parse_error(r"$", ParseError::FunctionUnusableMathMode);
}

#[test]
fn complains_about_missing_argument_at_end_of_input() {
    // 2\sqrt
    assert_parse_error(r"2\sqrt", ParseError::ExpectedGroup);
}

#[test]
fn complains_about_missing_argument_at_end_of_group() {
    // 1^{2\sqrt}
    assert_parse_error(r"1^{2\sqrt}", ParseError::ExpectedGroup);
}

#[test]
fn complains_about_functions_as_arguments() {
    // \sqrt\over2
    assert_parse_error(r"\sqrt\over2", ParseError::FunctionNoArguments);
}

#[test]
fn complains_about_undefined_control_sequence() {
    // \xyz
    assert_parse_error(r"\xyz", ParseError::UndefinedControlSequence(String::new()));
}

#[test]
fn complains_about_mismatched_verb_with_end_of_string() {
    // \verb|hello
    assert_fails(r"\verb|hello");
}

#[test]
fn complains_about_mismatched_verb_with_newline() {
    // \verb|hello\nworld|
    assert_fails("\\verb|hello\nworld|");
}

#[test]
fn complains_about_extra_closing_brace() {
    // {1+2}}
    assert_fails(r"{1+2}}");
}

#[test]
fn complains_about_extra_end() {
    // x\end{matrix}
    assert_fails(r"x\end{matrix}");
}

#[test]
fn complains_about_top_level_ampersand() {
    // 1&2
    assert_fails(r"1&2");
}

#[test]
fn rejects_missing_right() {
    // \left(1+2)
    assert_fails(r"\left(1+2)");
}

#[test]
fn rejects_incorrectly_scoped_right() {
    // {\left(1+2}\right)
    assert_fails(r"{\left(1+2}\right)");
}

#[test]
fn complains_about_missing_opening_brace_for_color() {
    // \textcolor#ffffff{text}
    assert_parse_error(r"\textcolor#ffffff{text}", ParseError::InvalidColor);
}

#[test]
fn complains_about_missing_opening_brace_for_size() {
    // \rule{1em}[2em]
    assert_parse_error(r"\rule{1em}[2em]", ParseError::InvalidSize);
}

#[test]
fn complains_about_missing_closing_brace_for_color() {
    // \textcolor{#ffffff{text}
    assert_fails(r"\textcolor{#ffffff{text}");
}

#[test]
fn complains_about_missing_closing_bracket_for_size() {
    // \rule[1em{2em}{3em}
    assert_fails(r"\rule[1em{2em}{3em}");
}

#[test]
fn complains_about_missing_bracket_at_end_for_size() {
    // \rule[1em
    assert_fails(r"\rule[1em");
}

#[test]
fn complains_about_missing_brace_at_end_for_color() {
    // \textcolor{#123456
    assert_fails(r"\textcolor{#123456");
}

#[test]
fn complains_about_missing_closing_brace_at_eof() {
    // \sqrt{2
    assert_fails(r"\sqrt{2");
}

#[test]
fn complains_about_missing_closing_bracket_at_eof() {
    // \sqrt[3
    assert_fails(r"\sqrt[3");
}

#[test]
fn complains_about_missing_bracket_before_group() {
    // \sqrt[3{2}
    assert_fails(r"\sqrt[3{2}");
}

#[test]
fn rejects_missing_end_in_array() {
    // \begin{matrix}1
    assert_fails(r"\begin{matrix}1");
}

#[test]
fn rejects_incorrectly_scoped_end_in_array() {
    // {\begin{matrix}1}\end{matrix}
    assert_fails(r"{\begin{matrix}1}\end{matrix}");
}

#[test]
fn rejects_unknown_column_types_in_array() {
    // \begin{array}{cba}\end{array}
    assert_fails(r"\begin{array}{cba}\end{array}");
}

#[test]
fn reject_invalid_opening_delimiters() {
    // \bigl 1 + 2 \bigr
    assert_fails(r"\bigl 1 + 2 \bigr");
}

#[test]
fn reject_invalid_closing_delimiters() {
    // \bigl(1+2\bigr=3
    assert_fails(r"\bigl(1+2\bigr=3");
}

#[test]
fn reject_group_opening_delimiters() {
    // \bigl{(}1+2\bigr)3
    assert_fails(r"\bigl{(}1+2\bigr)3");
}

#[test]
fn reject_group_closing_delimiters() {
    // \bigl(1+2\bigr{)}3
    assert_fails(r"\bigl(1+2\bigr{)}3");
}

#[test]
fn reject_invalid_environment_names() {
    // \begin x\end y
    assert_fails(r"\begin x\end y");
}

// =============================================================================
// Lexer Error Tests
// =============================================================================

#[test]
fn rejects_lone_surrogate_char() {
    // Note: Rust doesn't allow surrogate characters in string literals
    // This test would check for lone surrogates if we could encode them
    // Skipping this test as it's not representable in Rust source
    // The actual runtime handling of surrogates would need to be tested differently
}

#[test]
fn rejects_lone_backslash_at_end_of_input() {
    assert_fails(r"\");
}

#[test]
fn reject_3_digit_hex_without_hash() {
    // \textcolor{1a2}{foo}
    assert_parse_error(r"\textcolor{1a2}{foo}", ParseError::InvalidColor);
}

#[test]
fn reject_size_without_unit() {
    // \rule{0}{2em}
    assert_parse_error(r"\rule{0}{2em}", ParseError::InvalidSize);
}

#[test]
fn reject_size_with_bogus_unit() {
    // \rule{1au}{2em}
    assert_parse_error(r"\rule{1au}{2em}", ParseError::InvalidUnit);
}

#[test]
fn reject_size_without_number() {
    // \rule{em}{2em}
    assert_parse_error(r"\rule{em}{2em}", ParseError::InvalidSize);
}

// =============================================================================
// Unicode Accent Error Tests
// =============================================================================

#[test]
fn error_for_invalid_combining_characters() {
    // A with combining ogonek (not supported in KaTeX)
    assert_parse_error("A\u{0328}", ParseError::UnknownAccent);
}
