use aliter::{parse_tree, parser::ParserConfig, render_to_html_tree};

// Helper to check that parsing succeeds
fn assert_parses(expr: &str) {
    let conf = ParserConfig::default();
    let result = parse_tree(expr, conf);
    assert!(result.is_ok(), "Failed to parse: {}\nError: {:?}", expr, result.err());
}

// Helper for strict mode
fn assert_parses_strict(expr: &str) {
    let mut conf = ParserConfig::default();
    conf.strict = aliter::parser::StrictMode::Error;
    let result = parse_tree(expr, conf);
    assert!(result.is_ok(), "Failed to parse (strict): {}\nError: {:?}", expr, result.err());
}

// Helper to check that something doesn't parse in strict mode
fn assert_not_parses_strict(expr: &str) {
    let mut conf = ParserConfig::default();
    conf.strict = aliter::parser::StrictMode::Error;
    let result = parse_tree(expr, conf);
    assert!(result.is_err(), "Expected to fail in strict mode: {}", expr);
}

// Helper to render (builds HTML)
fn assert_builds(expr: &str) {
    let conf = ParserConfig::default();
    let _tree = render_to_html_tree(expr, conf);
    // If we get here without panicking, it built successfully
}

// =============================================================================
// Unicode Tests (from KaTeX unicode-spec.js)
// =============================================================================

// =============================================================================
// Latin-1 Tests
// =============================================================================

#[test]
fn should_build_latin1_inside_text() {
    assert_builds(r"\text{ÀÁÂÃÄÅÈÉÊËÌÍÎÏÑÒÓÔÕÖÙÚÛÜÝàáâãäåèéêëìíîïñòóôõöùúûüýÿÆÇÐØÞßæçðøþ}");
}

#[test]
fn should_not_parse_latin1_outside_text_with_strict() {
    // Test a sampling of Latin-1 characters
    let chars = vec!['À', 'Á', 'Â', 'Ã', 'Ä', 'Å', 'È', 'É', 'Ê', 'Ë', 'Ç', 'Ð', 'Þ', 'ç', 'þ'];
    for ch in chars {
        let expr = ch.to_string();
        assert_not_parses_strict(&expr);
    }
}

#[test]
fn should_build_latin1_outside_text_nonstrict() {
    assert_builds("ÀÁÂÃÄÅÈÉÊËÌÍÎÏÑÒÓÔÕÖÙÚÛÜÝàáâãäåèéêëìíîïñòóôõöùúûüýÿÇÐÞçðþ");
}

// =============================================================================
// Greek Letter Tests
// =============================================================================

#[test]
fn should_build_lowercase_greek_letters() {
    assert_builds("αβγδεϵζηθϑικλμνξοπϖρϱςστυφϕχψω");
}

#[test]
fn should_build_uppercase_greek_letters() {
    assert_builds("ΓΔΘΛΞΠΣΥΦΨΩ");
}

// =============================================================================
// Cyrillic Tests
// =============================================================================

#[test]
fn should_build_cyrillic_inside_text() {
    assert_builds(r"\text{БГДЖЗЙЛФЦШЫЮЯ}");
}

#[test]
fn should_build_cyrillic_outside_text_nonstrict() {
    assert_builds("БГДЖЗЙЛФЦШЫЮЯ");
}

#[test]
fn should_not_parse_cyrillic_outside_text_with_strict() {
    assert_not_parses_strict("БГДЖЗЙЛФЦШЫЮЯ");
}

// =============================================================================
// CJK (Chinese/Japanese/Korean) Tests
// =============================================================================

#[test]
fn should_build_cjk_inside_text() {
    assert_builds(r"\text{私はバナナです}");
    assert_builds(r"\text{여보세요}");
}

#[test]
fn should_build_cjk_outside_text_nonstrict() {
    assert_builds("私はバナナです");
    assert_builds("여보세요");
}

#[test]
fn should_not_parse_cjk_outside_text_with_strict() {
    assert_not_parses_strict("私はバナナです。");
    assert_not_parses_strict("여보세요");
}

// =============================================================================
// Devanagari Tests
// =============================================================================

#[test]
fn should_build_devanagari_inside_text() {
    assert_builds(r"\text{नमस्ते}");
}

#[test]
fn should_build_devanagari_outside_text_nonstrict() {
    assert_builds("नमस्ते");
}

#[test]
fn should_not_parse_devanagari_outside_text_with_strict() {
    assert_not_parses_strict("नमस्ते");
}

// =============================================================================
// Georgian Tests
// =============================================================================

#[test]
fn should_build_georgian_inside_text() {
    assert_builds(r"\text{გამარჯობა}");
}

#[test]
fn should_build_georgian_outside_text_nonstrict() {
    assert_builds("გამარჯობა");
}

#[test]
fn should_not_parse_georgian_outside_text_with_strict() {
    assert_not_parses_strict("გამარჯობა");
}

// =============================================================================
// Armenian Tests
// =============================================================================

#[test]
fn should_build_armenian_both_inside_and_outside_text() {
    assert_builds("ԱԲԳաբգ");
    assert_builds(r"\text{ԱԲԳաբգ}");
}

// =============================================================================
// Extended Latin Tests
// =============================================================================

#[test]
fn should_build_extended_latin_inside_text() {
    assert_builds(r"\text{ěščřžůřťďňőİı}");
}

#[test]
fn should_not_parse_extended_latin_outside_text_with_strict() {
    assert_not_parses_strict("ěščřžůřťďňőİı");
}

// =============================================================================
// Emoji Tests
// =============================================================================

#[test]
fn should_not_allow_emoji_in_strict_mode() {
    assert_not_parses_strict("✌");
    assert_not_parses_strict(r"\text{✌}");
}

#[test]
fn should_allow_emoji_outside_strict_mode() {
    // In non-strict mode, these should parse (possibly with warnings)
    assert_parses("✌");
    assert_parses(r"\text{✌}");
}

// =============================================================================
// Unicode Accent Equivalence Tests
// =============================================================================

// Note: These tests check that Unicode accented characters behave like
// their LaTeX accent command equivalents. This is complex to test at the
// parse tree level, so we just verify they parse/build correctly.

#[test]
fn latin1_accents_should_build_in_text_mode() {
    // Unicode versions
    assert_builds(r"\text{ÀÁÂÃÄÅÈÉÊËÌÍÎÏÑÒÓÔÕÖÙÚÛÜÝàáâãäåèéêëìíîïñòóôõöùúûüýÿÇç}");

    // LaTeX accent command versions (for reference - both should work)
    // Using regular strings since raw strings don't work with backslash escapes
    assert_builds("\\text{\\`A\\'A\\^A\\~A\\\"A\\r A\\`E\\'E\\^E\\\"E\\`I\\'I\\^I\\\"I\\~N\\`O\\'O\\^O\\~O\\\"O\\`U\\'U\\^U\\\"U\\'Y}");
}
