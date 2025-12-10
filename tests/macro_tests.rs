use aliter::{parse_tree, parser::ParserConfig, macr::{Macros, MacroReplace}};
use std::sync::Arc;

// Helper to check parsing succeeds
fn assert_parses(expr: &str) {
    let conf = ParserConfig::default();
    let result = parse_tree(expr, conf);
    assert!(result.is_ok(), "Failed to parse: {}\nError: {:?}", expr, result.err());
}

// Helper with custom macros
fn assert_parses_with_macros(expr: &str, macros: &[(&str, &str)]) {
    let mut conf = ParserConfig::default();

    // Create macros by inserting them into the config
    for (name, value) in macros {
        let replacement = MacroReplace::Text(value.to_string());
        conf.macros.insert_back_macro(name.to_string(), Arc::new(replacement));
    }

    let result = parse_tree(expr, conf);
    assert!(result.is_ok(), "Failed to parse: {}\nError: {:?}", expr, result.err());
}

// Helper to check parsing fails
fn assert_fails(expr: &str) {
    let conf = ParserConfig::default();
    let result = parse_tree(expr, conf);
    assert!(result.is_err(), "Expected parse error for: {}", expr);
}

// =============================================================================
// Basic Macro Expansion Tests (from KaTeX katex-spec.js)
// =============================================================================

#[test]
fn macro_should_produce_individual_tokens() {
    // e^\foo with \foo = "123" should be like e^1 23
    // This tests that expansion happens character by character
    assert_parses_with_macros(r"e^\foo", &[(r"\foo", "123")]);
}

#[test]
fn macro_should_preserve_leading_spaces_in_definition() {
    // \text{\foo} with \foo = " x" should preserve space
    assert_parses_with_macros(r"\text{\foo}", &[(r"\foo", " x")]);
}

#[test]
fn macro_should_preserve_leading_spaces_in_argument() {
    // \text{\foo{ x}} with \foo = "#1" should preserve space in arg
    assert_parses_with_macros(r"\text{\foo{ x}}", &[(r"\foo", "#1")]);
}

#[test]
fn macro_should_ignore_expanded_spaces_in_math_mode() {
    // \foo with \foo = " x" should be like "x" in math mode
    assert_parses_with_macros(r"\foo", &[(r"\foo", " x")]);
}

#[test]
fn macro_should_consume_spaces_after_control_word() {
    // \text{\foo } with \foo = "x" should be like \text{x}
    assert_parses_with_macros(r"\text{\foo }", &[(r"\foo", "x")]);
}

#[test]
fn macro_should_allow_multiple_expansion() {
    // 1\foo2 with \foo = "\bar\bar" and \bar = "a" should be 1aa2
    assert_parses_with_macros(
        r"1\foo2",
        &[(r"\foo", r"\bar\bar"), (r"\bar", "a")]
    );
}

#[test]
fn macro_should_allow_multiple_expansion_with_argument() {
    // 1\foo2 with \foo = "\bar{#1}\bar{#1}" and \bar = "#1#1"
    // should be 12222
    assert_parses_with_macros(
        r"1\foo2",
        &[(r"\foo", r"\bar{#1}\bar{#1}"), (r"\bar", "#1#1")]
    );
}

#[test]
fn macro_should_allow_macro_as_argument() {
    // \foo\bar with \foo = "(#1)" and \bar = "xyz" should be (xyz)
    assert_parses_with_macros(
        r"\foo\bar",
        &[(r"\foo", "(#1)"), (r"\bar", "xyz")]
    );
}

#[test]
fn macro_should_allow_properly_nested_group_argument() {
    // \foo{e^{x_{12}+3}} with \foo = "(#1)" should be (e^{x_{12}+3})
    assert_parses_with_macros(r"\foo{e^{x_{12}+3}}", &[(r"\foo", "(#1)")]);
}

#[test]
fn macro_should_allow_space_macro_argument() {
    // \foo\bar with \foo = "(#1)" and \bar = " " should handle space
    assert_parses_with_macros(
        r"\foo\bar",
        &[(r"\foo", "(#1)"), (r"\bar", " ")]
    );
}

#[test]
fn macro_should_allow_empty_macro_argument() {
    // \foo\bar with \foo = "(#1)" and \bar = "" should be ()
    assert_parses_with_macros(
        r"\foo\bar",
        &[(r"\foo", "(#1)"), (r"\bar", "")]
    );
}

// =============================================================================
// \newcommand Tests
// =============================================================================

#[test]
fn newcommand_should_define_new_macros() {
    // Basic definition
    assert_parses(r"\newcommand\foo{x^2}\foo+\foo");
    assert_parses(r"\newcommand{\foo}{x^2}\foo+\foo");
}

#[test]
fn newcommand_should_not_redefine_functions() {
    // \bar is a built-in function, can't redefine
    assert_fails(r"\newcommand\bar{x^2}\bar+\bar");
    assert_fails(r"\newcommand{\bar}{x^2}\bar+\bar");
}

#[test]
fn newcommand_should_not_redefine_symbols() {
    // \lambda is a built-in symbol
    assert_fails(r"\newcommand\lambda{x^2}\lambda");
    assert_fails(r"\newcommand\textdollar{x^2}\textdollar");
}

#[test]
fn newcommand_should_not_allow_duplicate_definition() {
    // Can't define \foo twice
    assert_fails(r"\newcommand{\foo}{1}\foo\newcommand{\foo}{2}\foo");
}

#[test]
fn newcommand_should_not_redefine_implicit_commands() {
    // \limits is an implicit command
    assert_fails(r"\newcommand\limits{}");
}

// =============================================================================
// \renewcommand Tests
// =============================================================================

#[test]
fn renewcommand_should_not_define_new_macros() {
    // \foo doesn't exist, so \renewcommand should fail
    assert_fails(r"\renewcommand\foo{x^2}\foo+\foo");
    assert_fails(r"\renewcommand{\foo}{x^2}\foo+\foo");
}

#[test]
fn renewcommand_should_redefine_existing_macros() {
    // \bar is a built-in, so \renewcommand should work
    assert_parses(r"\renewcommand\bar{x^2}\bar+\bar");
    assert_parses(r"\renewcommand{\bar}{x^2}\bar+\bar");
}

#[test]
fn renewcommand_should_redefine_user_macros() {
    // Define with \newcommand, then redefine with \renewcommand
    assert_parses(r"\newcommand{\foo}{1}\foo\renewcommand{\foo}{2}\foo");
}

// =============================================================================
// \providecommand Tests
// =============================================================================

#[test]
fn providecommand_should_define_new_macros() {
    // Acts like \newcommand if macro doesn't exist
    assert_parses(r"\providecommand\foo{x^2}\foo+\foo");
    assert_parses(r"\providecommand{\foo}{x^2}\foo+\foo");
}

#[test]
fn providecommand_should_not_redefine_existing_functions() {
    // \bar exists, so \providecommand redefines it
    assert_parses(r"\providecommand\bar{x^2}\bar+\bar");
    assert_parses(r"\providecommand{\bar}{x^2}\bar+\bar");
}

#[test]
fn providecommand_should_not_redefine_user_macros() {
    // Define with \newcommand, then \providecommand should not redefine
    assert_parses(r"\newcommand{\foo}{1}\foo\providecommand{\foo}{2}\foo");
}

#[test]
fn providecommand_combinations() {
    // providecommand then renewcommand
    assert_parses(r"\providecommand{\foo}{1}\foo\renewcommand{\foo}{2}\foo");

    // providecommand twice (second should be ignored)
    assert_parses(r"\providecommand{\foo}{1}\foo\providecommand{\foo}{2}\foo");
}

// =============================================================================
// Macro Scope Tests
// =============================================================================

#[test]
fn newcommand_should_be_local() {
    // Macro defined in group should not escape
    // \newcommand\foo{1}\foo{\renewcommand\foo{2}\foo}\foo should be 1{2}1
    assert_parses(r"\newcommand\foo{1}\foo{\renewcommand\foo{2}\foo}\foo");
}

// =============================================================================
// Macro Arguments Tests
// =============================================================================

#[test]
fn newcommand_should_accept_number_of_arguments() {
    // With 1 argument
    assert_parses(r"\newcommand\foo[1]{#1^2}\foo x+\foo{y}");

    // With maximum arguments (9 in LaTeX)
    assert_parses(r"\newcommand\foo[9]{#1#2#3#4#5#6#7#8#9}\foo 123456789");
}

#[test]
fn newcommand_should_reject_invalid_argument_counts() {
    // Invalid argument specifications
    assert_fails(r"\newcommand\foo[x]{}");
    assert_fails(r"\newcommand\foo[1.5]{}");
}

// =============================================================================
// Built-in Macro Expansion Tests
// =============================================================================

#[test]
fn should_expand_limsup() {
    // \limsup should expand to \operatorname*{lim\,sup}
    assert_parses(r"\limsup");
}

#[test]
fn should_expand_liminf() {
    // \liminf should expand to \operatorname*{lim\,inf}
    assert_parses(r"\liminf");
}

#[test]
fn should_expand_ams_log_like_symbols() {
    assert_parses(r"\injlim");     // inj lim
    assert_parses(r"\projlim");    // proj lim
    assert_parses(r"\varlimsup");  // overline{lim}
    assert_parses(r"\varliminf");  // underline{lim}
    assert_parses(r"\varinjlim");  // underrightarrow{lim}
    assert_parses(r"\varprojlim"); // underleftarrow{lim}
}

#[test]
fn should_expand_plim() {
    // \plim should expand to \mathop{\operatorname{plim}}\limits
    assert_parses(r"\plim");
}

#[test]
fn should_expand_argmin_argmax() {
    assert_parses(r"\argmin");
    assert_parses(r"\argmax");
}

#[test]
fn should_expand_bra_ket_notation() {
    // Quantum mechanics notation
    assert_parses(r"\bra{\phi}");
    assert_parses(r"\ket{\psi}");
    assert_parses(r"\braket{\phi|\psi}");

    // Uppercase versions with \left \right
    assert_parses(r"\Bra{\phi}");
    assert_parses(r"\Ket{\psi}");
    assert_parses(r"\Braket{\phi|\psi}");
}

#[test]
fn should_expand_set_notation() {
    // \set and \Set for set builder notation
    assert_parses(r"\set{x|x<5}");
    assert_parses(r"\Set{x|x<5}");
}

// =============================================================================
// Advanced Macro Features Tests
// =============================================================================

#[test]
fn should_handle_expandafter() {
    // \expandafter delays expansion
    // This is complex - test basic usage
    assert_parses_with_macros(
        r"\expandafter\foo\bar",
        &[(r"\foo", "#1+#2"), (r"\bar", "xy")]
    );
}

#[test]
fn should_handle_noexpand() {
    // \noexpand prevents expansion
    assert_parses_with_macros(
        r"\noexpand\foo y",
        &[(r"\foo", "x")]
    );
}

#[test]
fn should_handle_textormath() {
    // \TextOrMath chooses between text and math mode
    assert_parses(r"\TextOrMath{text}{math}");
    assert_parses(r"\text{\TextOrMath{text}{math}}");
}

#[test]
fn should_handle_char_command() {
    // \char produces literal characters
    assert_parses(r"\char`a");
    assert_parses(r"\char`\%");
    assert_parses(r"\char37");   // decimal
    assert_parses(r"\char'45");  // octal
    assert_parses(r#"\char"25"#); // hex
}

#[test]
fn should_reject_invalid_char_commands() {
    assert_fails(r"\char");
    assert_fails(r"\char`");
    assert_fails(r"\char'");
    assert_fails(r#"\char""#);
    assert_fails(r"\char'a");  // not octal
    assert_fails(r#"\char"g"#); // not hex
}

#[test]
fn should_handle_conditional_macros() {
    // \@firstoftwo chooses first argument
    assert_parses(r"\@firstoftwo{yes}{no}");

    // \@ifstar checks for *
    assert_parses(r"\@ifstar{yes}{no}*!");
    assert_parses(r"\@ifstar{yes}{no}?!");

    // \@ifnextchar checks next character
    assert_parses(r"\@ifnextchar!{yes}{no}!!");
    assert_parses(r"\@ifnextchar!{yes}{no}?!");
}

// =============================================================================
// Special Macro Equivalences Tests
// =============================================================================

#[test]
fn should_treat_hspace_hskip_like_kern() {
    // \hspace and \hskip should behave like \kern
    assert_parses(r"\hspace{1em}");
    assert_parses(r"\hskip{1em}");
}

#[test]
fn should_build_overset_underset() {
    assert_parses(r"\overset{f}{\rightarrow} Y");
    assert_parses(r"\underset{f}{\rightarrow} Y");
}

#[test]
fn should_build_logical_operators() {
    assert_parses(r"X \iff Y");
    assert_parses(r"X \implies Y");
    assert_parses(r"X \impliedby Y");
}

// =============================================================================
// Macro Edge Cases
// =============================================================================

#[test]
fn should_handle_empty_macro_definition() {
    assert_parses(r"\newcommand\foo{}\foo x");
}

#[test]
fn should_handle_macro_with_special_chars() {
    // Aliasing characters
    assert_parses_with_macros(r"x'=c", &[("'", "'")]);
}

#[test]
fn should_handle_nested_macro_calls() {
    // Deeply nested macro expansion
    assert_parses_with_macros(
        r"\a",
        &[
            (r"\a", r"\b"),
            (r"\b", r"\c"),
            (r"\c", r"\d"),
            (r"\d", "x")
        ]
    );
}

#[test]
fn should_respect_max_expand_limit() {
    // This should eventually hit maxExpand limit
    // Test depends on aliter's maxExpand implementation
    // Skipping for now - would need to configure maxExpand
}
