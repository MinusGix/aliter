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
// Lowercase Greek Letters
// =============================================================================

#[test]
fn should_parse_lowercase_greek_alpha_to_mu() {
    assert_parses(r"\alpha");
    assert_parses(r"\beta");
    assert_parses(r"\gamma");
    assert_parses(r"\delta");
    assert_parses(r"\epsilon");
    assert_parses(r"\varepsilon");
    assert_parses(r"\zeta");
    assert_parses(r"\eta");
    assert_parses(r"\theta");
    assert_parses(r"\vartheta");
    assert_parses(r"\iota");
    assert_parses(r"\kappa");
    assert_parses(r"\lambda");
    assert_parses(r"\mu");
}

#[test]
fn should_parse_lowercase_greek_nu_to_omega() {
    assert_parses(r"\nu");
    assert_parses(r"\xi");
    assert_parses(r"\omicron");
    assert_parses(r"\pi");
    assert_parses(r"\varpi");
    assert_parses(r"\rho");
    assert_parses(r"\varrho");
    assert_parses(r"\sigma");
    assert_parses(r"\varsigma");
    assert_parses(r"\tau");
    assert_parses(r"\upsilon");
    assert_parses(r"\phi");
    assert_parses(r"\varphi");
    assert_parses(r"\chi");
    assert_parses(r"\psi");
    assert_parses(r"\omega");
}

// =============================================================================
// Uppercase Greek Letters
// =============================================================================

#[test]
fn should_parse_uppercase_greek() {
    assert_parses(r"\Gamma");
    assert_parses(r"\Delta");
    assert_parses(r"\Theta");
    assert_parses(r"\Lambda");
    assert_parses(r"\Xi");
    assert_parses(r"\Pi");
    assert_parses(r"\Sigma");
    assert_parses(r"\Upsilon");
    assert_parses(r"\Phi");
    assert_parses(r"\Psi");
    assert_parses(r"\Omega");
}

// =============================================================================
// Binary Operators
// =============================================================================

#[test]
fn should_parse_basic_binary_operators() {
    assert_parses(r"+");
    assert_parses(r"-");
    assert_parses(r"*");
    assert_parses(r"/");
}

#[test]
fn should_parse_pm_and_mp() {
    assert_parses(r"\pm");
    assert_parses(r"\mp");
}

#[test]
fn should_parse_times_and_div() {
    assert_parses(r"\times");
    assert_parses(r"\div");
}

#[test]
fn should_parse_cdot_and_bullet() {
    assert_parses(r"\cdot");
    assert_parses(r"\bullet");
    assert_parses(r"\circ");
}

#[test]
fn should_parse_set_operators() {
    assert_parses(r"\cap");
    assert_parses(r"\cup");
    assert_parses(r"\setminus");
    assert_parses(r"\sqcap");
    assert_parses(r"\sqcup");
}

#[test]
fn should_parse_logical_operators() {
    assert_parses(r"\land");
    assert_parses(r"\lor");
    assert_parses(r"\wedge");
    assert_parses(r"\vee");
}

#[test]
fn should_parse_other_binary_operators() {
    assert_parses(r"\oplus");
    assert_parses(r"\ominus");
    assert_parses(r"\otimes");
    assert_parses(r"\oslash");
    assert_parses(r"\odot");
}

#[test]
fn should_parse_triangular_operators() {
    assert_parses(r"\triangleleft");
    assert_parses(r"\triangleright");
    assert_parses(r"\bigtriangleup");
    assert_parses(r"\bigtriangledown");
}

#[test]
fn should_parse_star_and_ast() {
    assert_parses(r"\star");
    assert_parses(r"\ast");
}

#[test]
fn should_parse_dagger() {
    assert_parses(r"\dagger");
    assert_parses(r"\ddagger");
}

// =============================================================================
// Relation Symbols
// =============================================================================

#[test]
fn should_parse_basic_relations() {
    assert_parses(r"=");
    assert_parses(r"<");
    assert_parses(r">");
    assert_parses(r"\leq");
    assert_parses(r"\geq");
    assert_parses(r"\le");
    assert_parses(r"\ge");
}

#[test]
fn should_parse_inequalities() {
    assert_parses(r"\neq");
    assert_parses(r"\ll");
    assert_parses(r"\gg");
    assert_parses(r"\leqslant");
    assert_parses(r"\geqslant");
}

#[test]
fn should_parse_equivalence_relations() {
    assert_parses(r"\equiv");
    assert_parses(r"\sim");
    assert_parses(r"\simeq");
    assert_parses(r"\cong");
    assert_parses(r"\approx");
}

#[test]
fn should_parse_proportionality() {
    assert_parses(r"\propto");
}

#[test]
fn should_parse_subset_relations() {
    assert_parses(r"\subset");
    assert_parses(r"\supset");
    assert_parses(r"\subseteq");
    assert_parses(r"\supseteq");
    assert_parses(r"\subsetneq");
    assert_parses(r"\supsetneq");
}

#[test]
fn should_parse_membership() {
    assert_parses(r"\in");
    assert_parses(r"\notin");
    assert_parses(r"\ni");
}

#[test]
fn should_parse_parallel_and_perp() {
    assert_parses(r"\parallel");
    assert_parses(r"\perp");
}

#[test]
fn should_parse_prec_and_succ() {
    assert_parses(r"\prec");
    assert_parses(r"\succ");
    assert_parses(r"\preceq");
    assert_parses(r"\succeq");
}

// =============================================================================
// Arrows
// =============================================================================

#[test]
fn should_parse_basic_arrows() {
    assert_parses(r"\leftarrow");
    assert_parses(r"\rightarrow");
    assert_parses(r"\uparrow");
    assert_parses(r"\downarrow");
    assert_parses(r"\updownarrow");
}

#[test]
fn should_parse_double_arrows() {
    assert_parses(r"\Leftarrow");
    assert_parses(r"\Rightarrow");
    assert_parses(r"\Uparrow");
    assert_parses(r"\Downarrow");
    assert_parses(r"\Updownarrow");
}

#[test]
fn should_parse_leftrightarrow() {
    assert_parses(r"\leftrightarrow");
    assert_parses(r"\Leftrightarrow");
}

#[test]
fn should_parse_long_arrows() {
    assert_parses(r"\longleftarrow");
    assert_parses(r"\longrightarrow");
    assert_parses(r"\Longleftarrow");
    assert_parses(r"\Longrightarrow");
    assert_parses(r"\longleftrightarrow");
    assert_parses(r"\Longleftrightarrow");
}

#[test]
fn should_parse_maps_to() {
    assert_parses(r"\mapsto");
    assert_parses(r"\longmapsto");
}

#[test]
fn should_parse_hooked_arrows() {
    assert_parses(r"\hookleftarrow");
    assert_parses(r"\hookrightarrow");
}

#[test]
fn should_parse_implied_arrows() {
    assert_parses(r"\implies");
    assert_parses(r"\impliedby");
    assert_parses(r"\iff");
}

#[test]
fn should_parse_nears() {
    assert_parses(r"\nearrow");
    assert_parses(r"\searrow");
    assert_parses(r"\swarrow");
    assert_parses(r"\nwarrow");
}

// =============================================================================
// Delimiters
// =============================================================================

#[test]
fn should_parse_parentheses() {
    assert_parses(r"(a+b)");
}

#[test]
fn should_parse_brackets() {
    assert_parses(r"[a,b]");
}

#[test]
fn should_parse_braces() {
    assert_parses(r"\{a,b\}");
}

#[test]
fn should_parse_angle_brackets() {
    assert_parses(r"\langle x\rangle");
}

#[test]
fn should_parse_floor_and_ceil() {
    assert_parses(r"\lfloor x\rfloor");
    assert_parses(r"\lceil x\rceil");
}

#[test]
fn should_parse_vertical_bars() {
    assert_parses(r"|x|");
    assert_parses(r"\|x\|");
}

// =============================================================================
// Large Operators
// =============================================================================

#[test]
fn should_parse_sum() {
    assert_parses(r"\sum");
    assert_parses(r"\sum_{i=1}^n");
}

#[test]
fn should_parse_prod() {
    assert_parses(r"\prod");
    assert_parses(r"\prod_{i=1}^n");
}

#[test]
fn should_parse_int() {
    assert_parses(r"\int");
    assert_parses(r"\int_a^b");
}

#[test]
fn should_parse_multiple_integrals() {
    assert_parses(r"\iint");
    assert_parses(r"\iiint");
    assert_parses(r"\oint");
}

#[test]
fn should_parse_limits() {
    assert_parses(r"\lim");
    assert_parses(r"\lim_{x\to\infty}");
    assert_parses(r"\limsup");
    assert_parses(r"\liminf");
}

#[test]
fn should_parse_bigcap_bigcup() {
    assert_parses(r"\bigcap");
    assert_parses(r"\bigcup");
}

#[test]
fn should_parse_bigvee_bigwedge() {
    assert_parses(r"\bigvee");
    assert_parses(r"\bigwedge");
}

#[test]
fn should_parse_bigoplus_bigotimes() {
    assert_parses(r"\bigoplus");
    assert_parses(r"\bigotimes");
}

// =============================================================================
// Special Symbols
// =============================================================================

#[test]
fn should_parse_infinity() {
    assert_parses(r"\infty");
}

#[test]
fn should_parse_nabla() {
    assert_parses(r"\nabla");
}

#[test]
fn should_parse_partial() {
    assert_parses(r"\partial");
}

#[test]
fn should_parse_hbar() {
    assert_parses(r"\hbar");
}

#[test]
fn should_parse_ell() {
    assert_parses(r"\ell");
}

#[test]
fn should_parse_aleph() {
    assert_parses(r"\aleph");
}

#[test]
fn should_parse_beth_gimel_daleth() {
    assert_parses(r"\beth");
    assert_parses(r"\gimel");
    assert_parses(r"\daleth");
}

#[test]
fn should_parse_emptyset() {
    assert_parses(r"\emptyset");
    assert_parses(r"\varnothing");
}

#[test]
fn should_parse_forall_exists() {
    assert_parses(r"\forall");
    assert_parses(r"\exists");
    assert_parses(r"\nexists");
}

#[test]
fn should_parse_neg_and_lnot() {
    assert_parses(r"\neg");
    assert_parses(r"\lnot");
}

#[test]
fn should_parse_top_and_bot() {
    assert_parses(r"\top");
    assert_parses(r"\bot");
}

#[test]
fn should_parse_angle() {
    assert_parses(r"\angle");
    assert_parses(r"\measuredangle");
    assert_parses(r"\sphericalangle");
}

#[test]
fn should_parse_prime() {
    assert_parses(r"x'");
    assert_parses(r"f''");
    assert_parses(r"g'''");
}

#[test]
fn should_parse_dots() {
    assert_parses(r"\ldots");
    assert_parses(r"\cdots");
    assert_parses(r"\vdots");
    assert_parses(r"\ddots");
}

// =============================================================================
// Function Names
// =============================================================================

#[test]
fn should_parse_trig_functions() {
    assert_parses(r"\sin");
    assert_parses(r"\cos");
    assert_parses(r"\tan");
    assert_parses(r"\cot");
    assert_parses(r"\sec");
    assert_parses(r"\csc");
}

#[test]
fn should_parse_inverse_trig() {
    assert_parses(r"\arcsin");
    assert_parses(r"\arccos");
    assert_parses(r"\arctan");
}

#[test]
fn should_parse_hyperbolic_trig() {
    assert_parses(r"\sinh");
    assert_parses(r"\cosh");
    assert_parses(r"\tanh");
    assert_parses(r"\coth");
}

#[test]
fn should_parse_log_functions() {
    assert_parses(r"\log");
    assert_parses(r"\ln");
    assert_parses(r"\lg");
}

#[test]
fn should_parse_exp() {
    assert_parses(r"\exp");
}

#[test]
fn should_parse_max_min() {
    assert_parses(r"\max");
    assert_parses(r"\min");
    assert_parses(r"\sup");
    assert_parses(r"\inf");
}

#[test]
fn should_parse_arg_functions() {
    assert_parses(r"\arg");
    assert_parses(r"\ker");
    assert_parses(r"\dim");
}

#[test]
fn should_parse_det_gcd() {
    assert_parses(r"\det");
    assert_parses(r"\gcd");
}

#[test]
fn should_parse_deg_hom() {
    assert_parses(r"\deg");
    assert_parses(r"\hom");
}

// =============================================================================
// Modular Arithmetic
// =============================================================================

#[test]
fn should_parse_mod_operators() {
    assert_parses(r"\bmod");
    assert_parses(r"\pmod{n}");
}

// =============================================================================
// Build Tests (Rendering)
// =============================================================================

#[test]
fn should_build_greek_letters() {
    assert_builds(r"\alpha\beta\gamma\delta");
}

#[test]
fn should_build_binary_operators() {
    assert_builds(r"a+b\times c\div d");
}

#[test]
fn should_build_relations() {
    assert_builds(r"x\leq y\geq z");
}

#[test]
fn should_build_arrows() {
    assert_builds(r"f:A\rightarrow B");
}

#[test]
fn should_build_large_operators() {
    assert_builds(r"\sum_{i=1}^n i^2");
}

// =============================================================================
// Complex Expressions with Symbols
// =============================================================================

#[test]
fn should_parse_set_notation() {
    assert_parses(r"\{x\in\mathbb{R}:x>0\}");
}

#[test]
fn should_parse_logical_expression() {
    assert_parses(r"\forall x\exists y(x<y)");
}

#[test]
fn should_parse_calculus_notation() {
    assert_parses(r"\int_0^\infty e^{-x}\,dx");
}

#[test]
fn should_parse_vector_calculus() {
    assert_parses(r"\nabla\times\vec{F}");
    assert_parses(r"\nabla\cdot\vec{E}");
}

#[test]
fn should_parse_quantum_notation() {
    assert_parses(r"\langle\psi|\hat{H}|\psi\rangle");
}

#[test]
fn should_build_complex_equation() {
    assert_builds(r"E=mc^2");
    assert_builds(r"e^{i\pi}+1=0");
    assert_builds(r"a^2+b^2=c^2");
}

// =============================================================================
// Special Symbol Combinations
// =============================================================================

#[test]
fn should_parse_symbols_with_subscripts_superscripts() {
    assert_parses(r"\sum_i");
    assert_parses(r"\prod^n");
    assert_parses(r"\int_a^b");
}

#[test]
fn should_parse_negated_relations() {
    assert_parses(r"\not=");
    assert_parses(r"\not\in");
    assert_parses(r"\not\equiv");
}

#[test]
fn should_parse_stacked_relations() {
    assert_parses(r"\stackrel{?}{=}");
    assert_parses(r"\overset{def}{=}");
}
