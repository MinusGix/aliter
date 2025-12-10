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
// Spacing Commands Tests
// =============================================================================

#[test]
fn should_parse_explicit_spacing_commands() {
    // Basic spacing commands
    assert_parses(r"a\,b");     // thin space
    assert_parses(r"a\:b");     // medium space
    assert_parses(r"a\;b");     // thick space
    assert_parses(r"a\ b");     // normal space
    assert_parses(r"a\!b");     // negative thin space
}

#[test]
fn should_parse_named_spacing_commands() {
    assert_parses(r"a\thinspace b");
    assert_parses(r"a\medspace b");
    assert_parses(r"a\thickspace b");
    assert_parses(r"a\enspace b");
    assert_parses(r"a\quad b");
    assert_parses(r"a\qquad b");
    assert_parses(r"a\negthinspace b");
    assert_parses(r"a\negmedspace b");
    assert_parses(r"a\negthickspace b");
}

#[test]
fn should_build_spacing_commands() {
    assert_builds(r"a\,b\:c\;d\ e\!f");
    assert_builds(r"\quad\qquad\enspace");
}

// =============================================================================
// Kern Tests
// =============================================================================

#[test]
fn should_parse_kern_with_various_units() {
    assert_parses(r"\kern1em");
    assert_parses(r"\kern1ex");
    assert_parses(r"\kern1mu");
    assert_parses(r"\kern1pt");
    assert_parses(r"\kern1cm");
    assert_parses(r"\kern1mm");
    assert_parses(r"\kern1in");
}

#[test]
fn should_parse_kern_with_negative_sizes() {
    assert_parses(r"\kern-1em");
    assert_parses(r"\kern-2.5ex");
}

#[test]
fn should_parse_kern_with_positive_sizes() {
    assert_parses(r"\kern+1em");
    assert_parses(r"\kern2.5ex");
}

#[test]
fn should_not_parse_kern_with_invalid_units() {
    assert_fails(r"\kern1badunit");
    assert_fails(r"\kern1");
}

#[test]
fn should_not_parse_kern_without_number() {
    assert_fails(r"\kernem");
}

// =============================================================================
// Hspace and Hskip Tests
// =============================================================================

#[test]
fn should_parse_hspace() {
    assert_parses(r"\hspace{1em}");
    assert_parses(r"\hspace{2.5ex}");
    assert_parses(r"\hspace{10pt}");
}

#[test]
fn should_parse_hskip() {
    assert_parses(r"\hskip{1em}");
    assert_parses(r"\hskip{2.5ex}");
}

#[test]
fn should_parse_hspace_star_variant() {
    assert_parses(r"\hspace*{1em}");
}

#[test]
fn should_parse_negative_hspace() {
    assert_parses(r"\hspace{-1em}");
    assert_parses(r"\hskip{-2ex}");
}

#[test]
fn should_build_hspace_and_hskip() {
    assert_builds(r"a\hspace{1em}b\hskip{2ex}c");
}

// =============================================================================
// Phantom and Smash Tests
// =============================================================================

#[test]
fn should_parse_phantom() {
    assert_parses(r"\phantom{x}");
    assert_parses(r"\phantom{x^2}");
    assert_parses(r"\phantom{\frac{a}{b}}");
}

#[test]
fn should_parse_vphantom() {
    assert_parses(r"\vphantom{x}");
    assert_parses(r"a\vphantom{A}b");
}

#[test]
fn should_parse_hphantom() {
    assert_parses(r"\hphantom{x}");
    assert_parses(r"a\hphantom{AAA}b");
}

#[test]
fn should_build_phantom_commands() {
    assert_builds(r"\phantom{x^2}+\vphantom{\frac{a}{b}}+\hphantom{text}");
}

#[test]
fn should_parse_smash() {
    assert_parses(r"\smash{x^2}");
    assert_parses(r"\smash[b]{x^2}");  // smash bottom
    assert_parses(r"\smash[t]{x^2}");  // smash top
}

// =============================================================================
// Lap Tests (llap, rlap, mathllap, mathrlap, mathclap)
// =============================================================================

#[test]
fn should_parse_llap_and_rlap() {
    assert_parses(r"\llap{text}");
    assert_parses(r"\rlap{text}");
}

#[test]
fn should_parse_math_lap_commands() {
    assert_parses(r"\mathllap{x}");
    assert_parses(r"\mathrlap{x}");
    assert_parses(r"\mathclap{x}");
}

#[test]
fn should_build_lap_commands() {
    assert_builds(r"\llap{(}a+b\rlap{)}");
    assert_builds(r"\mathllap{=}x\mathrlap{+}");
}

// =============================================================================
// Rule Tests
// =============================================================================

#[test]
fn should_parse_rule() {
    assert_parses(r"\rule{1em}{2em}");
    assert_parses(r"\rule{3pt}{4pt}");
}

#[test]
fn should_parse_rule_with_optional_raise() {
    assert_parses(r"\rule[1pt]{2em}{3em}");
    assert_parses(r"\rule[-0.5ex]{1em}{2ex}");
}

#[test]
fn should_not_parse_rule_without_units() {
    assert_fails(r"\rule{1}{2em}");
    assert_fails(r"\rule{1em}{2}");
}

#[test]
fn should_not_parse_rule_with_invalid_units() {
    assert_fails(r"\rule{1badunit}{2em}");
    assert_fails(r"\rule{1em}{2badunit}");
}

#[test]
fn should_build_rules() {
    assert_builds(r"\rule{2pt}{1em}\rule{1em}{2pt}");
    assert_builds(r"\rule[0.5ex]{3em}{0.4pt}");
}

// =============================================================================
// Tie (~) Tests
// =============================================================================

#[test]
fn should_parse_tie_in_math_mode() {
    assert_parses(r"a~b");
    assert_parses(r"x~y~z");
}

#[test]
fn should_parse_tie_in_text_mode() {
    assert_parses(r"\text{hello~world}");
    assert_parses(r"\text{a~b~c}");
}

#[test]
fn should_build_ties() {
    assert_builds(r"a~b~c");
    assert_builds(r"\text{hello~world}");
}

// =============================================================================
// Strut Tests
// =============================================================================

// Note: \strut is not supported by KaTeX (only \mathstrut is)

#[test]
fn should_parse_mathstrut() {
    assert_parses(r"\mathstrut");
    assert_parses(r"a\mathstrut b");
}

#[test]
fn should_build_mathstrut() {
    assert_builds(r"baseline\mathstrut test");
}

// =============================================================================
// Raisebox Tests
// =============================================================================

#[test]
fn should_parse_raisebox() {
    assert_parses(r"\raisebox{1em}{text}");
    assert_parses(r"\raisebox{-0.5ex}{lowered}");
}

#[test]
fn should_build_raisebox() {
    assert_builds(r"a\raisebox{1em}{raised}b");
    assert_builds(r"\raisebox{-1ex}{below}");
}

// =============================================================================
// Mspace Tests
// =============================================================================

// Note: \mspace is not supported by KaTeX (use \mkern or \hspace instead)

// =============================================================================
// Mkern Tests
// =============================================================================

#[test]
fn should_parse_mkern() {
    assert_parses(r"\mkern1mu");
    assert_parses(r"\mkern-2mu");
    assert_parses(r"\mkern3.5mu");
}

#[test]
fn should_build_mkern() {
    assert_builds(r"a\mkern3mu b");
}

// =============================================================================
// Combined Spacing Tests
// =============================================================================

#[test]
fn should_parse_complex_spacing_combinations() {
    assert_parses(r"a\,b\:c\;d\ e\!f\quad g\qquad h");
    assert_parses(r"\kern1em\hspace{2em}\mkern4mu");  // \mspace not implemented
    assert_parses(r"\phantom{x}\vphantom{y}\hphantom{z}");
}

#[test]
fn should_build_complex_spacing() {
    assert_builds(r"f(x)\,dx\quad\text{where}\ x\in\mathbb{R}");
    assert_builds(r"\int\!\!\!\int f(x,y)\,dx\,dy");
}

#[test]
fn should_parse_spacing_in_various_contexts() {
    assert_parses(r"\frac{a\,b}{c\:d}");
    assert_parses(r"x^{a\!b}_{ c\ d}");
    assert_parses(r"\sqrt{a\;b\quad c}");
}
