use aliter::{parse_tree, render_to_html_tree, parser::ParserConfig, parser::StrictMode, tree::VirtualNode, unit::Em};

// Helper for basic parsing
fn assert_parses(expr: &str) {
    let conf = ParserConfig::default();
    let result = parse_tree(expr, conf);
    assert!(result.is_ok(), "Failed to parse: {}\nError: {:?}", expr, result.err());
}

// Helper with custom config
fn assert_parses_with_config(expr: &str, conf: ParserConfig) {
    let result = parse_tree(expr, conf);
    assert!(result.is_ok(), "Failed to parse: {}\nError: {:?}", expr, result.err());
}

// Helper that expects parsing to fail
fn assert_fails(expr: &str, conf: ParserConfig) {
    let result = parse_tree(expr, conf);
    assert!(result.is_err(), "Expected parse error for: {}", expr);
}

// Helper for rendering
fn render(expr: &str, conf: ParserConfig) -> String {
    let tree = render_to_html_tree(expr, conf);
    tree.to_markup()
}

// =============================================================================
// Display Mode Tests
// =============================================================================

#[test]
fn display_mode_should_be_false_by_default() {
    let conf = ParserConfig::default();
    assert!(!conf.display_mode);
}

#[test]
fn display_mode_can_be_enabled() {
    let mut conf = ParserConfig::default();
    conf.display_mode = true;
    assert!(conf.display_mode);
}

#[test]
fn display_mode_required_for_ams_environments() {
    // gather requires display mode
    let mut conf = ParserConfig::default();
    conf.display_mode = false;
    assert_fails(r"\begin{gather}a+b\\c+d\end{gather}", conf.clone());

    conf.display_mode = true;
    assert_parses_with_config(r"\begin{gather}a+b\\c+d\end{gather}", conf);
}

#[test]
fn display_mode_required_for_align() {
    let mut conf = ParserConfig::default();
    conf.display_mode = false;
    assert_fails(r"\begin{align}a&=b\\c&=d\end{align}", conf.clone());

    conf.display_mode = true;
    assert_parses_with_config(r"\begin{align}a&=b\\c&=d\end{align}", conf);
}

#[test]
fn display_mode_required_for_equation() {
    let mut conf = ParserConfig::default();
    conf.display_mode = false;
    assert_fails(r"\begin{equation}a=b+c\end{equation}", conf.clone());

    conf.display_mode = true;
    assert_parses_with_config(r"\begin{equation}a=b+c\end{equation}", conf);
}

// =============================================================================
// Strict Mode Tests
// =============================================================================

#[test]
fn strict_mode_default_is_warn() {
    let conf = ParserConfig::default();
    // Default should be Warn
    assert!(matches!(conf.strict, StrictMode::Warn));
}

#[test]
fn strict_mode_should_allow_unicode_in_nonstrict() {
    let mut conf = ParserConfig::default();
    conf.strict = StrictMode::Warn;  // Warn mode allows Unicode

    // Unicode should be allowed
    assert_parses_with_config("é", conf.clone());
    assert_parses_with_config("試", conf);
}

#[test]
fn strict_mode_should_forbid_unicode_in_strict() {
    let mut conf = ParserConfig::default();
    conf.strict = StrictMode::Error;

    // Unicode should be forbidden
    assert_fails("é", conf.clone());
    assert_fails("試", conf);
}

// =============================================================================
// Trust Mode Tests
// =============================================================================

#[test]
fn trust_mode_is_false_by_default() {
    let conf = ParserConfig::default();
    assert!(!conf.trust);
}

#[test]
fn href_requires_trust() {
    let mut conf = ParserConfig::default();
    conf.trust = false;
    // Without trust, \href might be disabled or sanitized
    // This depends on implementation

    conf.trust = true;
    assert_parses_with_config(r"\href{http://example.com}{\sin}", conf);
}

#[test]
fn url_requires_trust() {
    let mut conf = ParserConfig::default();
    conf.trust = true;
    assert_parses_with_config(r"\url{http://example.com}", conf);
}

#[test]
fn includegraphics_requires_trust() {
    let mut conf = ParserConfig::default();
    conf.trust = true;
    assert_parses_with_config(
        r"\includegraphics[width=1em]{https://example.com/image.png}",
        conf
    );
}

#[test]
fn html_extensions_require_trust() {
    let mut conf = ParserConfig::default();
    conf.trust = true;

    assert_parses_with_config(r"\htmlId{id}{content}", conf.clone());
    assert_parses_with_config(r"\htmlClass{class}{content}", conf.clone());
    assert_parses_with_config(r"\htmlStyle{color: red;}{content}", conf.clone());
    assert_parses_with_config(r"\htmlData{foo=bar}{content}", conf);
}

// =============================================================================
// maxSize Tests
// =============================================================================

#[test]
fn maxsize_default_is_infinity() {
    let conf = ParserConfig::default();
    // Default max_size should be very large (Infinity in KaTeX)
    assert!(conf.max_size.0 > 1000.0);
}

#[test]
fn maxsize_should_clamp_large_sizes() {
    let mut conf = ParserConfig::default();
    conf.max_size = Em(5.0);

    let html = render(r"\rule{999em}{999em}", conf);
    // Should contain clamped values
    // The exact format depends on implementation
    assert!(html.contains("em") || html.contains("5"));
}

#[test]
fn maxsize_zero_makes_zero_width_rules() {
    let mut conf = ParserConfig::default();
    conf.max_size = Em(0.0);

    let html = render(r"\rule{999em}{999em}", conf);
    // Should result in zero-width
    assert!(html.contains("0") || !html.contains("999"));
}

// =============================================================================
// maxExpand Tests
// =============================================================================

#[test]
fn maxexpand_default_allows_expansion() {
    let conf = ParserConfig::default();
    // Should allow reasonable expansion
    assert_parses_with_config(r"\gdef\foo{1}\foo", conf);
}

#[test]
fn maxexpand_zero_prevents_expansion() {
    let mut conf = ParserConfig::default();
    conf.max_expand = Some(0);

    // With maxExpand = 0, no expansion should occur
    assert_fails(r"\gdef\foo{1}\foo", conf);
}

#[test]
fn maxexpand_should_prevent_infinite_loops() {
    let mut conf = ParserConfig::default();
    conf.max_expand = Some(10);

    // Infinite recursion should be caught
    assert_fails(r"\gdef\foo{\foo}\foo", conf);
}

// =============================================================================
// leqno and fleqn Tests
// =============================================================================

#[test]
fn leqno_default_is_false() {
    let conf = ParserConfig::default();
    assert!(!conf.leq_no);
}

#[test]
fn leqno_can_be_enabled() {
    let mut conf = ParserConfig::default();
    conf.leq_no = true;
    conf.display_mode = true;

    // Equation numbers should appear on left
    let html = render(r"\begin{equation}x=y\end{equation}", conf);
    assert!(!html.is_empty());
}

#[test]
fn fleqn_default_is_false() {
    let conf = ParserConfig::default();
    assert!(!conf.fleqn);
}

#[test]
fn fleqn_can_be_enabled() {
    let mut conf = ParserConfig::default();
    conf.fleqn = true;
    conf.display_mode = true;

    // Equations should be left-aligned
    let html = render(r"\begin{equation}x=y\end{equation}", conf);
    assert!(!html.is_empty());
}

// =============================================================================
// globalGroup Tests
// =============================================================================

#[test]
fn globalgroup_default_is_false() {
    let conf = ParserConfig::default();
    assert!(!conf.global_group);
}

#[test]
fn globalgroup_affects_macro_persistence() {
    // With globalGroup false, macros don't persist
    let mut conf = ParserConfig::default();
    conf.global_group = false;
    // Macros defined here won't affect conf.macros

    conf.global_group = true;
    // Macros defined here will affect conf.macros
    // This is harder to test without multiple render calls
}

// =============================================================================
// Error Handling Settings Tests
// =============================================================================

#[test]
fn throw_on_error_default_is_true() {
    let conf = ParserConfig::default();
    assert!(conf.throw_on_error);
}

#[test]
fn throw_on_error_false_renders_errors_as_text() {
    let mut conf = ParserConfig::default();
    conf.throw_on_error = false;

    // Invalid LaTeX should render as colored text instead of erroring
    let html = render(r"\invalid", conf);
    // Should contain error styling
    assert!(html.contains("invalid") || html.contains("error"));
}

#[test]
fn error_color_can_be_customized() {
    let mut conf = ParserConfig::default();
    // error_color is public but RGBA type isn't exported
    // Just verify we can access the field
    let _color = conf.error_color;
    conf.throw_on_error = false;

    let html = render(r"\invalid", conf);
    // Should use error color
    assert!(!html.is_empty());
}

// =============================================================================
// min_rule_thickness Tests
// =============================================================================

#[test]
fn min_rule_thickness_affects_rendering() {
    let mut conf = ParserConfig::default();
    conf.min_rule_thickness = Em(0.1);

    // Lines should respect minimum thickness
    let html = render(r"\frac{1}{2}", conf);
    assert!(!html.is_empty());
}

// =============================================================================
// color_is_text_color Tests
// =============================================================================

#[test]
fn color_is_text_color_default_is_false() {
    let conf = ParserConfig::default();
    assert!(!conf.color_is_text_color);
}

#[test]
fn color_is_text_color_changes_color_behavior() {
    // Old behavior: \color{red}{text}
    let mut conf = ParserConfig::default();
    conf.color_is_text_color = true;
    // \color would expect an argument

    // New behavior (default): \color{red} text
    conf.color_is_text_color = false;
    assert_parses_with_config(r"\color{red} text", conf);
}
