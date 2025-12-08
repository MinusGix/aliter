//! This is a conversion of the katex-spec.js file which tests KaTeX.

use crate::parser::ParserConfig;

#[cfg(test)]
mod tests {
    use crate::{
        expander::Mode,
        mathml::build_mathml,
        parse_node::{EqNoLoc, ParseNode},
        parse_tree,
        parser::{ParseError, ParserConfig, StrictMode},
        style::{StyleId, TEXT_STYLE},
        symbols::Atom,
        unit::Em,
        util::SourceLocation,
        Options,
    };

    enum TMode {
        Parse,
        Build,
    }
    impl TMode {
        fn apply(&self, expr: &str, conf: ParserConfig) -> Result<Vec<ParseNode>, ParseError> {
            parse_tree(expr, conf)
        }
    }

    #[derive(Debug)]
    enum ExpectedError {
        ParseError,
        ConsoleWarning,
    }

    fn strict_conf() -> ParserConfig {
        let mut conf = ParserConfig::default();
        conf.strict = StrictMode::Error;
        conf
    }

    fn default_options() -> Options {
        let mut options = Options::from_parser_conf(&ParserConfig::default());
        options.style = TEXT_STYLE;
        options.size = 5;
        options.max_size = Em(std::f64::INFINITY);
        options
    }

    #[track_caller]
    fn expect_katex(
        expr: &str,
        conf: ParserConfig,
        mode: TMode,
        is_not: Option<bool>,
        expected_error: Option<ExpectedError>,
    ) {
        let pass = expected_error.is_none();
        match mode.apply(expr, conf) {
            Ok(_) => expect_katex_fail(pass, expr, expected_error, None),
            Err(err) => match expected_error {
                Some(ExpectedError::ParseError) => {
                    expect_katex_fail(true, expr, expected_error, Some(err))
                }
                // TODO: console warnings?
                Some(ExpectedError::ConsoleWarning) => {
                    expect_katex_fail(false, expr, expected_error, Some(err))
                }
                None => expect_katex_fail(is_not.unwrap_or(false), expr, expected_error, Some(err)),
            },
        };
    }

    #[track_caller]
    fn expect_katex_fail(
        pass: bool,
        expr: &str,
        expected_error: Option<ExpectedError>,
        err: Option<ParseError>,
    ) {
        if !pass {
            let expected = match expected_error {
                Some(ExpectedError::ParseError) => "parse error",
                Some(ExpectedError::ConsoleWarning) => "console warning",
                None => "success",
            };
            panic!("Expected the expression {expr:?} to pass, but it failed: expected: {expected}. Error: {err:?}");
        }
    }

    #[track_caller]
    fn expect_equivalent(
        actual: &str,
        expected: &str,
        conf: ParserConfig,
        mode: TMode,
        expand: bool,
    ) {
        let actual_tree = mode.apply(actual, conf.clone()).unwrap();
        let expected_tree = mode.apply(expected, conf).unwrap();

        let pass = actual_tree.eq_no_loc(&expected_tree);
        if !pass {
            // TODO: log diff?
            panic!(
                "Expected the expression {actual:?} to be equivalent to {expected:?}, but it was not. Actual: {actual_tree:?}, Expected: {expected_tree:?}",
            );
        }
    }

    #[track_caller]
    fn to_parse(expr: &str, conf: ParserConfig) {
        expect_katex(expr, conf, TMode::Parse, None, None);
    }

    #[track_caller]
    fn to_not_parse(expr: &str, conf: ParserConfig) {
        expect_katex(expr, conf, TMode::Parse, Some(true), None);
    }

    #[track_caller]
    fn to_parse_like(actual: &str, expected: &str, conf: ParserConfig) {
        // TODO: expand
        expect_equivalent(actual, expected, conf, TMode::Parse, false)
    }

    #[track_caller]
    fn to_build_like(actual: &str, expected: &str, conf: ParserConfig) {
        expect_equivalent(actual, expected, conf, TMode::Build, false)
    }

    #[track_caller]
    fn to_build(expr: &str, conf: ParserConfig) {
        expect_katex(expr, conf, TMode::Build, None, None);
    }

    #[track_caller]
    fn to_not_build(expr: &str, conf: ParserConfig) {
        expect_katex(expr, conf, TMode::Build, Some(true), None);
    }

    #[test]
    fn a_parser() {
        // should not fail on an empty string
        to_parse("", strict_conf());

        // should ignore whitespace
        to_parse_like("    x    y    ", "xy", strict_conf());

        // should ignore whitespace in atom
        to_parse_like("    x   ^ y    ", "x^y", strict_conf());
    }

    #[test]
    fn an_ord_parser() {
        let expr = "1234|/@.\"`abcdefgzABCDEFGZ";

        // it should not fail
        to_parse(expr, ParserConfig::default());

        {
            let parse = parse_tree(expr, ParserConfig::default()).unwrap();

            // should build a list of only ords
            for group in &parse {
                if !matches!(
                    group,
                    ParseNode::OrdGroup(_) | ParseNode::TextOrd(_) | ParseNode::MathOrd(_)
                ) {
                    panic!("Expected ordgroup, got {:?}", group);
                }
            }

            // Should parse the right number of ords
            assert_eq!(parse.len(), expr.len());
        }
    }

    #[test]
    fn a_bin_parser() {
        let expr = r#"+-*\cdot\pm\div"#;

        // it should not fail
        to_parse(expr, ParserConfig::default());

        {
            let parse = parse_tree(expr, ParserConfig::default()).unwrap();

            for group in parse {
                let ParseNode::Atom(atom) = group else {
                    panic!("Expected atom, got {:?}", group);
                };

                assert_eq!(atom.family, Atom::Bin, "Expected binary operation atom");
            }
        }
    }

    #[test]
    fn a_rel_parser() {
        let expr = r#"=<>\leq\geq\neq\nleq\ngeq\cong"#;
        let not_expr = r#"\not=\not<\not>\not\leq\not\geq\not\in"#;

        // should not fail
        to_parse(expr, ParserConfig::default());
        to_parse(not_expr, ParserConfig::default());

        {
            let parse = parse_tree(expr, ParserConfig::default()).unwrap();

            for group in parse {
                let group = if let ParseNode::HtmlMathml(group) = group {
                    assert_eq!(group.html.len(), 1);
                    group.html.into_iter().nth(0).unwrap()
                } else {
                    group
                };

                match group {
                    ParseNode::MClass(mclass) => {
                        assert_eq!(mclass.m_class, "mrel");
                    }
                    ParseNode::Atom(atom) => {
                        assert_eq!(atom.family, Atom::Rel, "Expected relation atom");
                    }
                    _ => panic!("Expected MClass or Atom ParseNode, got {:?}", group),
                }
            }
        }
    }

    #[test]
    fn a_mathinner_parser() {
        // should not fail
        to_parse(
            "\\mathinner{\\langle{\\psi}\\rangle}",
            ParserConfig::default(),
        );
        to_parse(
            "\\frac 1 {\\mathinner{\\langle{\\psi}\\rangle}}",
            ParserConfig::default(),
        );

        {
            // Should return one group, not a fragment
            let contents = "\\mathinner{\\langle{\\psi}\\rangle}";
            let parsed = parse_tree(contents, ParserConfig::default()).unwrap();
            let mml = build_mathml(&parsed, contents, &default_options(), false, false);
            assert_eq!(mml.children.len(), 1);
        }
    }

    #[test]
    fn a_punct_parser() {
        let expr = ",;";

        // should not fail
        to_parse(expr, strict_conf());

        {
            // should build a list of puncts
            let parse = parse_tree(expr, ParserConfig::default()).unwrap();

            for group in parse {
                let ParseNode::Atom(atom) = group else {
                    panic!("Expected atom, got {:?}", group);
                };

                assert_eq!(atom.family, Atom::Punct, "Expected punctuation atom");
            }
        }
    }

    #[test]
    fn an_open_parser() {
        let expr = "([";

        // should not fail
        to_parse(expr, ParserConfig::default());

        {
            // should build a list of opens
            let parse = parse_tree(expr, ParserConfig::default()).unwrap();

            for group in parse {
                let ParseNode::Atom(atom) = group else {
                    panic!("Expected atom, got {:?}", group);
                };

                assert_eq!(atom.family, Atom::Open, "Expected open atom");
            }
        }
    }

    #[test]
    fn a_close_parser() {
        let expr = ")]?!";

        // should not fail
        to_parse(expr, ParserConfig::default());

        {
            // should build a list of closes
            let parse = parse_tree(expr, ParserConfig::default()).unwrap();

            for group in parse {
                let ParseNode::Atom(atom) = group else {
                    panic!("Expected atom, got {:?}", group);
                };

                assert_eq!(atom.family, Atom::Close, "Expected close atom");
            }
        }
    }

    #[test]
    fn a_katex_parser() {
        // should not fail
        to_parse("\\KaTeX", ParserConfig::default());
    }

    #[test]
    fn a_subscript_and_superscript_parser() {
        // should not fail on superscripts
        to_parse("x^2", ParserConfig::default());

        // should not fail on subscripts
        to_parse("x_3", ParserConfig::default());

        // should not fail on both subscripts and superscripts
        to_parse("x^2_3", ParserConfig::default());
        to_parse("x_2^3", ParserConfig::default());

        // should not fail when there is no nucleus
        to_parse("^3", ParserConfig::default());
        to_parse("^3+", ParserConfig::default());
        to_parse("_2", ParserConfig::default());
        to_parse("^3_2", ParserConfig::default());
        to_parse("_2^3", ParserConfig::default());

        // should produce supsubs for superscript
        {
            let parse = parse_tree("x^2", ParserConfig::default()).unwrap();
            let ParseNode::SupSub(supsub) = &parse[0] else {
                panic!("Expected SupSub, got {:?}", parse[0]);
            };

            assert!(supsub.base.is_some());
            assert!(supsub.sup.is_some());
            assert!(supsub.sub.is_none());
        }

        // should produce supsubs for subscript
        {
            let parse = parse_tree("x_3", ParserConfig::default()).unwrap();
            let ParseNode::SupSub(supsub) = &parse[0] else {
                panic!("Expected SupSub, got {:?}", parse[0]);
            };

            assert!(supsub.base.is_some());
            assert!(supsub.sub.is_some());
            assert!(supsub.sup.is_none());
        }

        // should produce supsubs for ^_
        {
            let parse = parse_tree("x^2_3", ParserConfig::default()).unwrap();
            let ParseNode::SupSub(supsub) = &parse[0] else {
                panic!("Expected SupSub, got {:?}", parse[0]);
            };

            assert!(supsub.base.is_some());
            assert!(supsub.sup.is_some());
            assert!(supsub.sub.is_some());
        }

        // should produce supsubs for _^
        {
            let parse = parse_tree("x_3^2", ParserConfig::default()).unwrap();
            let ParseNode::SupSub(supsub) = &parse[0] else {
                panic!("Expected SupSub, got {:?}", parse[0]);
            };

            assert!(supsub.base.is_some());
            assert!(supsub.sup.is_some());
            assert!(supsub.sub.is_some());
        }

        // should produce the same thing regardless of order
        to_parse_like("x^2_3", "x_3^2", ParserConfig::default());

        // should not parse double subscripts or superscripts
        to_not_parse("x^x^x", ParserConfig::default());
        to_not_parse("x_x_x", ParserConfig::default());
        to_not_parse("x_x^x_x", ParserConfig::default());
        to_not_parse("x_x^x^x", ParserConfig::default());
        to_not_parse("x^x_x_x", ParserConfig::default());
        to_not_parse("x^x_x^x", ParserConfig::default());

        // should work correctly with {}s
        to_parse("x^{2+3}", ParserConfig::default());
        to_parse("x_{3-2}", ParserConfig::default());
        to_parse("x^{2+3}_3", ParserConfig::default());
        to_parse("x^2_{3-2}", ParserConfig::default());
        to_parse("x^{2+3}_{3-2}", ParserConfig::default());
        to_parse("x_{3-2}^{2+3}", ParserConfig::default());
        to_parse("x_3^{2+3}", ParserConfig::default());
        to_parse("x_{3-2}^2", ParserConfig::default());

        // should work with nested super/subscripts
        to_parse("x^{x^x}", ParserConfig::default());
        to_parse("x^{x_x}", ParserConfig::default());
        to_parse("x_{x^x}", ParserConfig::default());
        to_parse("x_{x_x}", ParserConfig::default());

        // should work with Unicode (sub|super)script characters
        to_parse("A² + B²⁺³ + ¹²C + E₂³ + F₂₊₃", ParserConfig::default());
    }

    #[test]
    fn a_subscript_and_superscript_tree_builder() {
        // should not fail when there is no nucleus
        to_build("^3", ParserConfig::default());
        to_build("_3", ParserConfig::default());
        to_build("^3_2", ParserConfig::default());
        to_build("_2^3", ParserConfig::default());
    }

    #[test]
    fn a_parser_with_limit_controls() {
        // should fail when the limit control is not preceded by an op node
        to_not_parse("3\nolimits_2^2", ParserConfig::default());
        to_not_parse("\\sqrt\\limits_2^2", ParserConfig::default());
        to_not_parse("45 +\nolimits 45", ParserConfig::default());

        // should parse when the limit control directly follows an op node
        to_parse("\\int\\limits_2^2 3", ParserConfig::default());
        to_parse("\\sum\\nolimits_3^4 4", ParserConfig::default());

        // should parse when the limit control is in the sup/sub area of an op node
        to_parse("\\int_2^2\\limits", ParserConfig::default());
        to_parse("\\int^2\\nolimits_2", ParserConfig::default());
        to_parse("\\int_2\\limits^2", ParserConfig::default());

        // should allow multiple limit controls in the sup/sub area of an op node
        to_parse("\\int_2\\nolimits^2\\limits 3", ParserConfig::default());
        to_parse("\\int\\nolimits\\limits_2^2", ParserConfig::default());
        to_parse("\\int\\limits\\limits\\limits_2^2", ParserConfig::default());

        // should have the rightmost limit control determine the limits property of the preceding op node
        {
            let parse = parse_tree("\\int\\nolimits\\limits_2^2", ParserConfig::default()).unwrap();
            let ParseNode::SupSub(sup_sub) = &parse[0] else {
                panic!("Expected SupSub, got {:?}", parse[0]);
            };

            let Some(ParseNode::Op(base)) = sup_sub.base.as_deref() else {
                panic!("Expected Op, got {:?}", sup_sub.base);
            };

            assert!(base.limits);
        }

        {
            let parse = parse_tree("\\int\\limits_2\\nolimits^2", ParserConfig::default()).unwrap();
            let ParseNode::SupSub(sup_sub) = &parse[0] else {
                panic!("Expected SupSub, got {:?}", parse[0]);
            };

            let Some(ParseNode::Op(base)) = sup_sub.base.as_deref() else {
                panic!("Expected Op, got {:?}", sup_sub.base);
            };

            assert!(!base.limits);
        }
    }

    #[test]
    fn a_group_parser() {
        // should not fail
        to_parse("{xy}", ParserConfig::default());

        // should produce a single ord
        {
            let parse = parse_tree("{xy}", ParserConfig::default()).unwrap();
            let ParseNode::OrdGroup(ord) = &parse[0] else {
                panic!("Expected Ord, got {:?}", parse[0]);
            };

            assert!(!ord.body.is_empty());
        }
    }

    #[test]
    fn a_begingroup_endgroup_parser() {
        // should not fail when properly closed
        to_parse(r"\begingroup xy \endgroup", ParserConfig::default());

        // should fail when it is mismatched
        to_not_parse(r"\begingroup xy", ParserConfig::default());
        to_not_parse(r"\begingroup xy }", ParserConfig::default());

        // should produce a semi-simple ord group
        let parse = parse_tree(r"\begingroup xy \endgroup", ParserConfig::default()).unwrap();
        assert_eq!(parse.len(), 1);
        let ParseNode::OrdGroup(ord) = &parse[0] else {
            panic!("Expected OrdGroup, got {:?}", parse[0]);
        };
        assert!(!ord.body.is_empty());
        assert_eq!(ord.semi_simple, Some(true));

        // should not affect spacing in math mode
        to_parse_like(
            r"\begingroup x+ \endgroup y",
            "x+y",
            ParserConfig::default(),
        );
    }

    #[test]
    fn an_implicit_group_parser() {
        // should not fail
        to_parse(r"\Large x", ParserConfig::default());
        to_parse(r"abc {abc \Large xyz} abc", ParserConfig::default());

        // should produce a single sizing object
        let parse = parse_tree(r"\Large abc", ParserConfig::default()).unwrap();
        assert_eq!(parse.len(), 1);
        let ParseNode::Styling(styling) = &parse[0] else {
            panic!("Expected Styling node, got {:?}", parse[0]);
        };
        assert!(!styling.body.is_empty());

        // should apply only after the function
        let parse = parse_tree(r"a \Large abc", ParserConfig::default()).unwrap();
        assert_eq!(parse.len(), 2);
        let ParseNode::Styling(styling) = &parse[1] else {
            panic!("Expected Styling node, got {:?}", parse[1]);
        };
        assert_eq!(styling.body.len(), 3);

        // should stop at the ends of groups
        let parse = parse_tree(r"a { b \Large c } d", ParserConfig::default()).unwrap();
        let ParseNode::OrdGroup(group) = &parse[1] else {
            panic!("Expected inner group, got {:?}", parse[1]);
        };
        let ParseNode::Styling(styling) = &group.body[1] else {
            panic!("Expected Styling node, got {:?}", group.body[1]);
        };
        assert_eq!(styling.body.len(), 1);

        // optional-group variants: just ensure they parse
        to_parse(r"\sqrt[\small 3]{x}", ParserConfig::default());
        to_parse(r"\sqrt[\color{red} 3]{x}", ParserConfig::default());
        to_parse(r"\sqrt[\textstyle 3]{x}", ParserConfig::default());
        to_parse(r"\sqrt[\tt 3]{x}", ParserConfig::default());
    }

    #[test]
    fn a_function_parser() {
        // should parse 0/1/2-argument functions when arguments are present
        to_parse(r"\div", ParserConfig::default());
        to_parse(r"\blue x", ParserConfig::default());
        to_parse(r"\frac 1 2", ParserConfig::default());

        // should reject missing arguments
        to_not_parse(r"\blue", ParserConfig::default());
        to_not_parse(r"\frac", ParserConfig::default());
        to_not_parse(r"\frac 1", ParserConfig::default());

        // should not parse function immediately followed by text
        to_not_parse(r"\redx", ParserConfig::default());

        // should parse when followed by numbers or spacing commands
        to_parse(r"\frac12", ParserConfig::default());
        to_parse(r"\;x", ParserConfig::default());
    }

    #[test]
    fn a_genfrac_builder() {
        // should not fail for common fraction helpers
        for expr in [
            r"\frac{x}{y}",
            r"\dfrac{x}{y}",
            r"\tfrac{x}{y}",
            r"\cfrac{x}{y}",
            r"\genfrac ( ] {0.06em}{0}{a}{b+c}",
            r"\genfrac ( ] {0.8pt}{}{a}{b+c}",
            r"\genfrac {} {} {0.8pt}{}{a}{b+c}",
            r"\genfrac [ {} {0.8pt}{}{a}{b+c}",
        ] {
            to_build(expr, ParserConfig::default());
        }
    }

    #[test]
    fn a_infix_builder() {
        // should build infix fractions of various styles
        for expr in [r"a \over b", r"a \atop b", r"a \choose b", r"a \brace b", r"a \brack b"] {
            to_build(expr, ParserConfig::default());
        }
    }

    #[test]
    fn a_text_parser_extended() {
        let text_expr = r"\text{a b}";
        let no_brace_text = r"\text x";
        let nested_text = r"\text{a {b} \blue{c} \textcolor{#fff}{x} \llap{x}}";
        let space_text = r"\text{  a \  }";
        let leading_space_text = r"\text {moo}";
        let bad_text = r"\text{a b%}";
        let bad_function = r"\text{\sqrt{x}}";
        let math_token_after_text = r"\text{sin}^2";

        // should parse basic forms
        to_parse(text_expr, ParserConfig::default());
        to_parse(no_brace_text, ParserConfig::default());
        to_parse(nested_text, ParserConfig::default());
        to_parse(space_text, ParserConfig::default());
        to_parse(leading_space_text, ParserConfig::default());

        // should produce text nodes with textord children
        let parse = parse_tree(text_expr, ParserConfig::default()).unwrap();
        let ParseNode::Text(text) = &parse[0] else {
            panic!("Expected Text node, got {:?}", parse[0]);
        };
        assert!(!text.body.is_empty());
        let first = &text.body[0];
        if let ParseNode::TextOrd(ord) = first {
            assert_eq!(ord.text, "a");
        }

        // should not parse malformed text or math-only functions inside text
        to_not_parse(bad_text, ParserConfig::default());
        to_not_parse(bad_function, ParserConfig::default());

        // should allow math tokens after text blocks
        to_parse(math_token_after_text, ParserConfig::default());
    }

    #[test]
    fn an_over_brace_brack_parser() {
        let simple_over = r"1 \over x";
        let complex_over = r"1+2i \over 3+4i";
        let brace_frac = r"a+b \brace c+d";
        let brack_frac = r"a+b \brack c+d";

        // should not fail
        to_parse(simple_over, ParserConfig::default());
        to_parse(complex_over, ParserConfig::default());
        to_parse(brace_frac, ParserConfig::default());
        to_parse(brack_frac, ParserConfig::default());

        // should produce genfrac nodes with numer/denom present
        for expr in [simple_over, complex_over, brace_frac, brack_frac] {
            let parse = parse_tree(expr, ParserConfig::default()).unwrap();
            let ParseNode::GenFrac(genfrac) = &parse[0] else {
                panic!("Expected GenFrac, got {:?}", parse[0]);
            };
            // presence checks
            let _ = &genfrac.numer;
            let _ = &genfrac.denom;
        }

        // check delimiters for brace/brack
        let parse = parse_tree(brace_frac, ParserConfig::default()).unwrap();
        let ParseNode::GenFrac(genfrac) = &parse[0] else {
            panic!("Expected GenFrac, got {:?}", parse[0]);
        };
        assert!(genfrac.left_delim.is_some());
        assert!(genfrac.right_delim.is_some());

        let parse = parse_tree(brack_frac, ParserConfig::default()).unwrap();
        let ParseNode::GenFrac(genfrac) = &parse[0] else {
            panic!("Expected GenFrac, got {:?}", parse[0]);
        };
        assert!(genfrac.left_delim.is_some());
        assert!(genfrac.right_delim.is_some());

        // empty numerator / denominator
        let parse = parse_tree(r"\over x", ParserConfig::default()).unwrap();
        assert!(matches!(parse[0], ParseNode::GenFrac(_)));
        let parse = parse_tree(r"1 \over", ParserConfig::default()).unwrap();
        assert!(matches!(parse[0], ParseNode::GenFrac(_)));

        // should reject multiple infix fractions in one group
        to_not_parse(r"1 \over 2 + 3 \over 4", ParserConfig::default());
        to_not_parse(r"1 \over 2 \choose 3", ParserConfig::default());
    }

    #[test]
    fn a_supsub_left_right_nucleus_parser() {
        // should parse juxtaposition in superscripts/subscripts with left/right delimiters
        to_parse(r"x^\left(3\right)4", ParserConfig::default());
        to_parse(r"x_\left(3\right)4", ParserConfig::default());
        to_parse(r"x_\left(3^\left(4\right)\right)5", ParserConfig::default());
    }

    #[test]
    fn an_over_under_parser() {
        // should parse \overline and \underline
        to_parse(r"\overline{x}", ParserConfig::default());
        to_parse(r"\underline{x}", ParserConfig::default());
    }

    #[test]
    fn a_phantom_parser() {
        // should parse \phantom, \hphantom, \vphantom
        to_parse(r"\phantom{x}", ParserConfig::default());
        to_parse(r"\hphantom{x}", ParserConfig::default());
        to_parse(r"\vphantom{x}", ParserConfig::default());
    }

    #[test]
    fn a_color_parser() {
        let mut conf = ParserConfig::default();
        conf.color_is_text_color = false;

        // should parse \color with implicit body
        to_parse(r"\color{blue} x + y", conf.clone());
        to_parse(r"\textcolor{#fff}{x}", conf);
    }

    #[test]
    fn a_kern_parser() {
        // should parse explicit kerns
        to_parse(r"x\kern1em y", ParserConfig::default());
        to_parse(r"x\mkern1mu y", ParserConfig::default());
    }

    #[test]
    fn a_rule_parser() {
        // should parse \rule with width/height
        to_parse(r"\rule{1em}{2em}", ParserConfig::default());
    }

    #[test]
    fn a_text_mode_switch_parser() {
        // should parse \text / \textrm and consume the argument group
        to_parse(r"\text{abc}", ParserConfig::default());
        to_parse(r"\textrm{abc}", ParserConfig::default());
    }

    #[test]
    fn an_overbrace_underbrace_parser() {
        to_parse(r"\overbrace{abc}^{note}", ParserConfig::default());
        to_parse(r"\underbrace{abc}_{note}", ParserConfig::default());
    }

    #[test]
    fn a_text_font_parser() {
        to_parse(r"\textit{abc}", ParserConfig::default());
        to_parse(r"\textbf{abc}", ParserConfig::default());
        to_parse(r"\textsf{abc}", ParserConfig::default());
        to_parse(r"\texttt{abc}", ParserConfig::default());
        to_parse(r"\textnormal{abc}", ParserConfig::default());
    }

    #[test]
    fn a_math_font_parser() {
        to_parse(r"\mathbb{ABC}", ParserConfig::default());
        to_parse(r"\mathfrak{ABC}", ParserConfig::default());
        to_parse(r"\mathcal{ABC}", ParserConfig::default());
        to_parse(r"\mathsf{ABC}", ParserConfig::default());
    }

    #[test]
    fn a_spacing_parser() {
        to_parse(r"a\,b\\;c\\!d", ParserConfig::default());
    }

    #[test]
    fn a_text_color_parser() {
        to_parse(r"\color{red}{x}", ParserConfig::default());
        to_parse(r"\textcolor{blue}{y}", ParserConfig::default());
    }

    #[test]
    fn a_delimiter_sizing_parser() {
        to_parse(r"\left( \frac{1}{2} \right)", ParserConfig::default());
        to_parse(r"\bigl( x \bigr)", ParserConfig::default());
        to_parse(r"\Bigl( x \Bigr)", ParserConfig::default());
    }

    #[test]
    fn a_sqrt_parser() {
        to_parse(r"\sqrt{2}", ParserConfig::default());
        to_parse(r"\sqrt[3]{x}", ParserConfig::default());
    }

    #[test]
    fn a_binom_parser() {
        to_parse(r"\binom{n}{k}", ParserConfig::default());
    }

    #[test]
    fn a_frac_parser() {
        to_parse(r"\frac{a}{b}", ParserConfig::default());
        to_parse(r"\dfrac{1}{2}", ParserConfig::default());
    }

    #[test]
    fn a_left_right_parser() {
        to_parse(r"\left( x + y \right)", ParserConfig::default());
        to_parse(r"\left[ \frac{1}{2} \right]", ParserConfig::default());
    }

    #[test]
    fn a_matrix_parser() {
        to_parse(
            r"\begin{matrix} a & b \\ c & d \end{matrix}",
            ParserConfig::default(),
        );
        to_parse(
            r"\begin{pmatrix} 1 & 2 \\ 3 & 4 \end{pmatrix}",
            ParserConfig::default(),
        );
        to_parse(
            r"\begin{cases} a & b \\ c & d \end{cases}",
            ParserConfig::default(),
        );
    }

    #[test]
    fn a_phantom_sizing_parser() {
        to_parse(r"\hphantom{abc}", ParserConfig::default());
        to_parse(r"\vphantom{xyz}", ParserConfig::default());
    }

    #[test]
    fn a_rule_color_parser() {
        to_parse(r"\color{green}{\rule{1em}{1em}}", ParserConfig::default());
    }

    #[test]
    fn an_accent_parser() {
        to_parse(r"\hat{x}", ParserConfig::default());
        to_parse(r"\bar{y}", ParserConfig::default());
        to_parse(r"\vec{v}", ParserConfig::default());
    }

    #[test]
    fn a_sizing_parser() {
        to_parse(r"{\Huge A}", ParserConfig::default());
        to_parse(r"{\tiny B}", ParserConfig::default());
    }

    #[test]
    fn an_arrow_parser() {
        to_parse(r"\xleftarrow{abc}", ParserConfig::default());
        to_parse(r"\xrightarrow{def}", ParserConfig::default());
    }

    #[test]
    fn a_tie_parser() {
        let math_tie = "a~b";
        let text_tie = r"\text{a~ b}";

        // should parse ties in math mode
        to_parse(math_tie, ParserConfig::default());

        // should parse ties in text mode
        to_parse(text_tie, ParserConfig::default());

        // should produce spacing in math mode
        {
            let parse = parse_tree(math_tie, ParserConfig::default()).unwrap();
            let ParseNode::Spacing(_) = &parse[1] else {
                panic!("Expected Spacing, got {:?}", parse[1]);
            };
        }

        // should produce spacing in text mode
        {
            let parse = parse_tree(text_tie, ParserConfig::default()).unwrap();
            let ParseNode::Text(text) = &parse[0] else {
                panic!("Expected Text, got {:?}", parse[0]);
            };
            let body = &text.body;

            let ParseNode::Spacing(_) = &body[1] else {
                panic!("Expected Spacing, got {:?}", body[1]);
            };
        }

        // should not contract with spaces in text mode
        {
            let parse = parse_tree(text_tie, ParserConfig::default()).unwrap();
            let ParseNode::Text(text) = &parse[0] else {
                panic!("Expected Text, got {:?}", parse[0]);
            };
            let body = &text.body;

            let ParseNode::Spacing(_) = &body[2] else {
                panic!("Expected Spacing, got {:?}", body[2]);
            };
        }
    }

    #[test]
    fn an_overline_parser() {
        let overline = r"\overline{x}";

        // should not fail
        to_parse(overline, ParserConfig::default());

        // should produce an overline
        {
            let parse = parse_tree(overline, ParserConfig::default()).unwrap();
            let ParseNode::Overline(_) = &parse[0] else {
                panic!("Expected Overline, got {:?}", parse[0]);
            };
        }
    }

    #[test]
    fn an_lap_parser() {
        // should not fail on a text argument
        to_parse(r"\rlap{\,/}{=}", ParserConfig::default());
        to_parse(r"\mathrlap{\,/}{=}", ParserConfig::default());
        to_parse(r"{=}\llap{/\,}", ParserConfig::default());
        to_parse(r"{=}\mathllap{/\,}", ParserConfig::default());
        to_parse(r"\sum_{\clap{ABCDEFG}}", ParserConfig::default());
        to_parse(r"\sum_{\mathclap{ABCDEFG}}", ParserConfig::default());

        // should not fail if math version is used
        to_parse(r"\mathrlap{\frac{a}{b}}{=}", ParserConfig::default());
        to_parse(r"{=}\mathllap{\frac{a}{b}}", ParserConfig::default());
        to_parse(r"\sum_{\mathclap{\frac{a}{b}}}", ParserConfig::default());

        // should fail on math if AMS version is used
        to_not_parse(r"\rlap{\frac{a}{b}}{=}", ParserConfig::default());
        to_not_parse(r"{=}\llap{\frac{a}{b}}", ParserConfig::default());
        to_not_parse(r"\sum_{\clap{\frac{a}{b}}}", ParserConfig::default());

        // should produce a lap
        {
            let parse = parse_tree(r"\mathrlap{\,/}", ParserConfig::default()).unwrap();
            let ParseNode::Lap(_) = &parse[0] else {
                panic!("Expected Lap, got {:?}", parse[0]);
            };
        }
    }

    #[test]
    fn left_right_builder() {
        // should build "..." like "..."
        let cases = [
            (r"\left\langle \right\rangle", r"\left< \right>"),
            (r"\left\langle \right\rangle", "\\left\u{27e8} \\right\u{27e9}"),
            (r"\left\lparen \right\rparen", r"\left( \right)"),
        ];

        for (actual, expected) in cases {
            to_build_like(actual, expected, ParserConfig::default());
        }
    }

    #[test]
    fn a_tex_compliant_parser() {
        // should work
        to_parse(r"\frac 2 3", ParserConfig::default());

        // should fail if there are not enough arguments
        let missing_groups = [
            r"\frac{x}",
            r"\textcolor{#fff}",
            r"\rule{1em}",
            r"\llap",
            r"\bigl",
            r"\text",
        ];

        for expr in missing_groups {
            to_not_parse(expr, ParserConfig::default());
        }

        // should fail when there are missing sup/subscripts
        to_not_parse("x^", ParserConfig::default());
        to_not_parse("x_", ParserConfig::default());

        // should fail when arguments require arguments
        to_not_parse(r"\frac \frac x y z", ParserConfig::default());
    }

    #[test]
    fn a_style_change_parser() {
        // should not fail
        to_parse(r"\displaystyle x", ParserConfig::default());
        to_parse(r"\textstyle x", ParserConfig::default());
        to_parse(r"\scriptstyle x", ParserConfig::default());
        to_parse(r"\scriptscriptstyle x", ParserConfig::default());

        // should produce the correct style
        {
            let parse = parse_tree(r"\displaystyle x", ParserConfig::default()).unwrap();
            let ParseNode::Styling(styling) = &parse[0] else {
                panic!("Expected Styling, got {:?}", parse[0]);
            };
            assert_eq!(styling.style, StyleId::D);
        }
        {
            let parse = parse_tree(r"\scriptscriptstyle x", ParserConfig::default()).unwrap();
            let ParseNode::Styling(styling) = &parse[0] else {
                panic!("Expected Styling, got {:?}", parse[0]);
            };
            assert_eq!(styling.style, StyleId::SS);
        }

        // should only change the style within its group
        {
            let text = r"a b { c d \displaystyle e f } g h";
            let parse = parse_tree(text, ParserConfig::default()).unwrap();

            // parse[2] is the group { ... }
            let ParseNode::OrdGroup(group) = &parse[2] else {
                panic!("Expected OrdGroup, got {:?}", parse[2]);
            };
            
            let display_node = &group.body[2];
            let ParseNode::Styling(styling) = display_node else {
                panic!("Expected Styling at index 2, got {:?}", display_node);
            };
            assert_eq!(styling.style, StyleId::D);
            
            let display_body = &styling.body;
            assert_eq!(display_body.len(), 2); // e, f
            // check text "e"
            let ParseNode::TextOrd(ord) = &display_body[0] else {
                 panic!("Expected TextOrd, got {:?}", display_body[0]);
            };
            assert_eq!(ord.text, "e");
        }
    }

    #[test]
    fn an_op_symbol_builder() {
        to_build(r"\int_i^n", ParserConfig::default());
        to_build(r"\iint_i^n", ParserConfig::default());
        to_build(r"\iiint_i^n", ParserConfig::default());
        to_build(r"\int\nolimits_i^n", ParserConfig::default());
        to_build(r"\iint\nolimits_i^n", ParserConfig::default());
        to_build(r"\iiint\nolimits_i^n", ParserConfig::default());
        to_build(r"\oint_i^n", ParserConfig::default());
        to_build(r"\oiint_i^n", ParserConfig::default());
        to_build(r"\oiiint_i^n", ParserConfig::default());
        to_build(r"\oint\nolimits_i^n", ParserConfig::default());
        to_build(r"\oiint\nolimits_i^n", ParserConfig::default());
        to_build(r"\oiiint\nolimits_i^n", ParserConfig::default());
    }

    #[test]
    fn a_font_parser() {
        // should parse \mathrm, \mathbb, \mathit, and \mathnormal
        to_parse(r"\mathrm x", ParserConfig::default());
        to_parse(r"\mathbb x", ParserConfig::default());
        to_parse(r"\mathit x", ParserConfig::default());
        to_parse(r"\mathnormal x", ParserConfig::default());
        to_parse(r"\mathrm {x + 1}", ParserConfig::default());
        to_parse(r"\mathbb {x + 1}", ParserConfig::default());
        to_parse(r"\mathit {x + 1}", ParserConfig::default());
        to_parse(r"\mathnormal {x + 1}", ParserConfig::default());

        // should parse \mathcal and \mathfrak
        to_parse(r"\mathcal{ABC123}", ParserConfig::default());
        to_parse(r"\mathfrak{abcABC123}", ParserConfig::default());

        // should produce the correct fonts
        {
            let parse = parse_tree(r"\mathbb x", ParserConfig::default()).unwrap();
            let ParseNode::Font(font) = &parse[0] else {
                panic!("Expected Font, got {:?}", parse[0]);
            };
            assert_eq!(font.font, "mathbb");
        }
        {
            let parse = parse_tree(r"\mathrm x", ParserConfig::default()).unwrap();
            let ParseNode::Font(font) = &parse[0] else {
                panic!("Expected Font, got {:?}", parse[0]);
            };
            assert_eq!(font.font, "mathrm");
        }
        {
             let parse = parse_tree(r"\mathit x", ParserConfig::default()).unwrap();
            let ParseNode::Font(font) = &parse[0] else {
                panic!("Expected Font, got {:?}", parse[0]);
            };
            assert_eq!(font.font, "mathit");
        }
        {
            let parse = parse_tree(r"\mathnormal x", ParserConfig::default()).unwrap();
            let ParseNode::Font(font) = &parse[0] else {
                panic!("Expected Font, got {:?}", parse[0]);
            };
            assert_eq!(font.font, "mathnormal");
        }
        {
            let parse = parse_tree(r"\mathcal C", ParserConfig::default()).unwrap();
            let ParseNode::Font(font) = &parse[0] else {
                panic!("Expected Font, got {:?}", parse[0]);
            };
            assert_eq!(font.font, "mathcal");
        }
        {
            let parse = parse_tree(r"\mathfrak C", ParserConfig::default()).unwrap();
            let ParseNode::Font(font) = &parse[0] else {
                panic!("Expected Font, got {:?}", parse[0]);
            };
            assert_eq!(font.font, "mathfrak");
        }

        // should parse nested font commands
        {
            let parse = parse_tree(r"\mathbb{R \neq \mathrm{R}}", ParserConfig::default()).unwrap();
            let ParseNode::Font(font) = &parse[0] else {
                panic!("Expected Font, got {:?}", parse[0]);
            };
            assert_eq!(font.font, "mathbb");
            
            let ParseNode::OrdGroup(group) = font.body.as_ref() else {
                 panic!("Expected OrdGroup body, got {:?}", font.body);
            };
            
            // R, \neq, \mathrm{R}
            assert_eq!(group.body.len(), 3);
            assert!(matches!(group.body[0], ParseNode::MathOrd(_)));
            // index 2 is \mathrm{R}
             let ParseNode::Font(inner_font) = &group.body[2] else {
                panic!("Expected Font at index 2, got {:?}", group.body[2]);
            };
            assert_eq!(inner_font.font, "mathrm");
        }

        // should work with \textcolor
        {
            let parse = parse_tree(r"\textcolor{blue}{\mathbb R}", ParserConfig::default()).unwrap();
            let ParseNode::Color(color) = &parse[0] else {
                 panic!("Expected Color, got {:?}", parse[0]);
            };
             // TODO: verify color value "blue" if accessible
            
            let body = &color.body;
            assert_eq!(body.len(), 1);
            let ParseNode::Font(font) = &body[0] else {
                panic!("Expected Font inside Color, got {:?}", body[0]);
            };
            assert_eq!(font.font, "mathbb");
        }
    }

    #[test]
    fn a_pmb_builder() {
        to_build(r"\pmb{\mu}", ParserConfig::default());
        to_build(r"\pmb{=}", ParserConfig::default());
        to_build(r"\pmb{+}", ParserConfig::default());
        to_build(r"\pmb{\frac{x^2}{x_1}}", ParserConfig::default());
        to_build(r"\pmb{}", ParserConfig::default());
        to_parse_like(r"\def\x{1}\pmb{\x\def\x{2}}", r"\pmb{1}", ParserConfig::default());
    }

    #[test]
    fn a_raise_parser() {
        // should parse and build text in \raisebox
        to_build(r"\raisebox{5pt}{text}", strict_conf());
        to_build(r"\raisebox{-5pt}{text}", strict_conf());

        // should parse and build math in non-strict \vcenter
        // TODO: nonstrict
        to_build(r"\vcenter{\frac a b}", ParserConfig::default());

        // should fail to parse math in \raisebox
        // TODO: nonstrict
        // expect(r"\raisebox{5pt}{\frac a b}").not.toParse(nonstrictSettings);
        
        // should fail to parse math in an \hbox
        // expect(r"\hbox{\frac a b}").not.toParse(nonstrictSettings);

        // should fail to build, given an unbraced length
        to_not_build(r"\raisebox5pt{text}", strict_conf());
        to_not_build(r"\raisebox-5pt{text}", strict_conf());

        // should build math in an hbox when math mode is set
        to_build(r"a + \vcenter{\hbox{$\frac{\frac a b}c$}}", strict_conf());
    }

    #[test]
    fn a_comment_parser() {
        // should parse comments at the end of a line
        to_parse("a^2 + b^2 = c^2 % Pythagoras' Theorem\n", ParserConfig::default());

        // should parse comments at the start of a line
        to_parse("% comment\n", ParserConfig::default());

        // should parse multiple lines of comments in a row
        to_parse("% comment 1\n% comment 2\n", ParserConfig::default());

        // should parse comments between subscript and superscript
        to_parse_like("x_3 %comment\n^2", "x_3^2", ParserConfig::default());
        to_parse_like("x^ %comment\n{2}", "x^{2}", ParserConfig::default());
        to_parse_like("x^ %comment\n\\frac{1}{2}", r"x^\frac{1}{2}", ParserConfig::default());

        // should parse comments in size and color groups
        to_parse("\\kern{1 %kern\nem}", ParserConfig::default());
        to_parse("\\kern1 %kern\nem", ParserConfig::default());
        to_parse("\\color{#f00%red\n}", ParserConfig::default());

        // should parse comments before an expression
        to_parse_like("%comment\n{2}", "{2}", ParserConfig::default());

        // should parse comments before and between \hline
        to_parse("\\begin{matrix}a&b\\\\ %hline\n\\hline %hline\n\\hline c&d\\end{matrix}", ParserConfig::default());

        // should parse comments in the macro definition
        to_parse_like("\\def\\foo{1 %}\n2}\n\\foo", "12", ParserConfig::default());

        // should not expand nor ignore spaces after a command sequence in a comment
        to_parse_like("\\def\\foo{1\n2}\nx %\\foo\n", "x", ParserConfig::default());

        // should not parse a comment without newline in strict mode
        to_not_parse("x%y", strict_conf());
        // expect`x%y`.toParse(nonstrictSettings);

        // should not produce or consume space
        to_parse_like("\\text{hello% comment 1\nworld}", r"\text{helloworld}", ParserConfig::default());
        to_parse_like("\\text{hello% comment\n\nworld}", r"\text{hello world}", ParserConfig::default());

        // should not include comments in the output
        to_parse_like("5 % comment\n", "5", ParserConfig::default());
    }

    #[test]
    fn a_parse_tree_generator() {
        // generates a tree
        // stripPositions() corresponds to EqNoLoc in our context
        to_parse_like(r"\sigma^2", r"\sigma^2", ParserConfig::default());
    }

    #[test]
    fn an_accent_builder() {
        // should not fail
        to_build(r"\vec{x}", ParserConfig::default());
        to_build(r"\vec{x}^2", ParserConfig::default());
        to_build(r"\vec{x}_2", ParserConfig::default());
        to_build(r"\vec{x}_2^2", ParserConfig::default());

        // Skipping class checks for now.
    }

    #[test]
    fn a_stretchy_and_shifty_accent_builder() {
        // should not fail
        to_build(r"\widehat{AB}", ParserConfig::default());
        to_build(r"\widecheck{AB}", ParserConfig::default());
        to_build(r"\widehat{AB}^2", ParserConfig::default());
        to_build(r"\widehat{AB}_2", ParserConfig::default());
        to_build(r"\widehat{AB}_2^2", ParserConfig::default());

        // Skipping class checks for now.
    }

    #[test]
    fn a_stretchy_and_non_shifty_accent_builder() {
        // should not fail
        to_build(r"\overrightarrow{AB}", ParserConfig::default());
        to_build(r"\overrightarrow{AB}^2", ParserConfig::default());
        to_build(r"\overrightarrow{AB}_2", ParserConfig::default());
        to_build(r"\overrightarrow{AB}_2^2", ParserConfig::default());

        // Skipping class checks for now.
    }

    #[test]
    fn an_under_accent_parser() {
        // should not fail
        to_parse(r"\underrightarrow{x}", ParserConfig::default());
        to_parse(r"\underrightarrow{x^2}", ParserConfig::default());
        to_parse(r"\underrightarrow{x}^2", ParserConfig::default());
        to_parse(r"\underrightarrow x", ParserConfig::default());

        // should produce accentUnder
        {
            let parse = parse_tree(r"\underrightarrow x", ParserConfig::default()).unwrap();
            let ParseNode::AccentUnder(_) = &parse[0] else {
                panic!("Expected AccentUnder, got {:?}", parse[0]);
            };
        }

        // should be grouped more tightly than supsubs
        {
            let parse = parse_tree(r"\underrightarrow x^2", ParserConfig::default()).unwrap();
            let ParseNode::SupSub(_) = &parse[0] else {
                panic!("Expected SupSub, got {:?}", parse[0]);
            };
        }
    }

    #[test]
    fn an_under_accent_builder() {
        // should not fail
        to_build(r"\underrightarrow{x}", ParserConfig::default());
        to_build(r"\underrightarrow{x}^2", ParserConfig::default());
        to_build(r"\underrightarrow{x}_2", ParserConfig::default());
        to_build(r"\underrightarrow{x}_2^2", ParserConfig::default());

        // Skipping class checks for now.
    }

    #[test]
    fn an_extensible_arrow_parser() {
        // should not fail
        to_parse(r"\xrightarrow{x}", ParserConfig::default());
        to_parse(r"\xrightarrow{x^2}", ParserConfig::default());
        to_parse(r"\xrightarrow{x}^2", ParserConfig::default());
        to_parse(r"\xrightarrow x", ParserConfig::default());
        to_parse(r"\xrightarrow[under]{over}", ParserConfig::default());

        // should produce xArrow
        {
            let parse = parse_tree(r"\xrightarrow x", ParserConfig::default()).unwrap();
            let ParseNode::XArrow(_) = &parse[0] else {
                panic!("Expected XArrow, got {:?}", parse[0]);
            };
        }

        // should be grouped more tightly than supsubs
        {
            let parse = parse_tree(r"\xrightarrow x^2", ParserConfig::default()).unwrap();
            let ParseNode::SupSub(_) = &parse[0] else {
                panic!("Expected SupSub, got {:?}", parse[0]);
            };
        }
    }

    #[test]
    fn an_extensible_arrow_builder() {
        // should not fail
        to_build(r"\xrightarrow{x}", ParserConfig::default());
        to_build(r"\xrightarrow{x}^2", ParserConfig::default());
        to_build(r"\xrightarrow{x}_2", ParserConfig::default());
        to_build(r"\xrightarrow{x}_2^2", ParserConfig::default());
        to_build(r"\xrightarrow[under]{over}", ParserConfig::default());

        // Skipping class checks for now.
    }

    #[test]
    fn a_horizontal_brace_parser() {
        // should not fail
        to_parse(r"\overbrace{x}", ParserConfig::default());
        to_parse(r"\overbrace{x^2}", ParserConfig::default());
        to_parse(r"\overbrace{x}^2", ParserConfig::default());
        to_parse(r"\overbrace x", ParserConfig::default());
        to_parse(r"\underbrace{x}_2", ParserConfig::default());
        to_parse(r"\underbrace{x}_2^2", ParserConfig::default());

        // should produce horizBrace
        {
            let parse = parse_tree(r"\overbrace x", ParserConfig::default()).unwrap();
            let ParseNode::HorizBrace(_) = &parse[0] else {
                panic!("Expected HorizBrace, got {:?}", parse[0]);
            };
        }

        // should be grouped more tightly than supsubs
        {
            let parse = parse_tree(r"\overbrace x^2", ParserConfig::default()).unwrap();
            let ParseNode::SupSub(_) = &parse[0] else {
                panic!("Expected SupSub, got {:?}", parse[0]);
            };
        }
    }

    #[test]
    fn a_horizontal_brace_builder() {
        // should not fail
        to_build(r"\overbrace{x}", ParserConfig::default());
        to_build(r"\overbrace{x}^2", ParserConfig::default());
        to_build(r"\underbrace{x}_2", ParserConfig::default());
        to_build(r"\underbrace{x}_2^2", ParserConfig::default());

        // Skipping class checks for now.
    }

    #[test]
    fn a_boxed_parser() {
        // should not fail
        to_parse(r"\boxed{x}", ParserConfig::default());
        to_parse(r"\boxed{x^2}", ParserConfig::default());
        to_parse(r"\boxed{x}^2", ParserConfig::default());
        to_parse(r"\boxed x", ParserConfig::default());

        // should produce enclose
        {
            let parse = parse_tree(r"\boxed x", ParserConfig::default()).unwrap();
            let ParseNode::Enclose(_) = &parse[0] else {
                panic!("Expected Enclose, got {:?}", parse[0]);
            };
        }
    }

    #[test]
    fn a_boxed_builder() {
        // should not fail
        to_build(r"\boxed{x}", ParserConfig::default());
        to_build(r"\boxed{x}^2", ParserConfig::default());
        to_build(r"\boxed{x}_2", ParserConfig::default());
        to_build(r"\boxed{x}_2^2", ParserConfig::default());

        // Skipping class checks for now.
    }

    #[test]
    fn an_fbox_parser_unlike_a_boxed_parser() {
        // should fail when given math
        to_not_parse(r"\fbox{\frac a b}", ParserConfig::default());
    }

    #[test]
    fn a_colorbox_parser() {
        // should not fail, given a text argument
        to_parse(r"\colorbox{red}{a b}", ParserConfig::default());
        to_parse(r"\colorbox{red}{x}^2", ParserConfig::default());
        to_parse(r"\colorbox{red} x", ParserConfig::default());

        // should fail, given a math argument
        to_not_parse(r"\colorbox{red}{\alpha}", ParserConfig::default());
        to_not_parse(r"\colorbox{red}{\frac{a}{b}}", ParserConfig::default());

        // should parse a color
        to_parse(r"\colorbox{red}{a b}", ParserConfig::default());
        to_parse(r"\colorbox{#197}{a b}", ParserConfig::default());
        to_parse(r"\colorbox{#1a9b7c}{a b}", ParserConfig::default());

        // should produce enclose
        {
            let parse = parse_tree(r"\colorbox{red} x", ParserConfig::default()).unwrap();
            let ParseNode::Enclose(_) = &parse[0] else {
                panic!("Expected Enclose, got {:?}", parse[0]);
            };
        }
    }

    #[test]
    fn a_colorbox_builder() {
        // should not fail
        to_build(r"\colorbox{red}{a b}", ParserConfig::default());
        to_build(r"\colorbox{red}{a b}^2", ParserConfig::default());
        to_build(r"\colorbox{red} x", ParserConfig::default());

        // Skipping class checks for now.
    }

    #[test]
    fn an_fcolorbox_parser() {
        // should not fail, given a text argument
        to_parse(r"\fcolorbox{blue}{yellow}{a b}", ParserConfig::default());
        to_parse(r"\fcolorbox{blue}{yellow}{x}^2", ParserConfig::default());
        to_parse(r"\fcolorbox{blue}{yellow} x", ParserConfig::default());

        // should fail, given a math argument
        to_not_parse(r"\fcolorbox{blue}{yellow}{\alpha}", ParserConfig::default());
        to_not_parse(r"\fcolorbox{blue}{yellow}{\frac{a}{b}}", ParserConfig::default());

        // should parse a color
        to_parse(r"\fcolorbox{blue}{yellow}{a b}", ParserConfig::default());
        to_parse(r"\fcolorbox{blue}{#197}{a b}", ParserConfig::default());
        to_parse(r"\fcolorbox{blue}{#1a9b7c}{a b}", ParserConfig::default());

        // should produce enclose
        {
            let parse = parse_tree(r"\fcolorbox{blue}{yellow} x", ParserConfig::default()).unwrap();
            let ParseNode::Enclose(_) = &parse[0] else {
                panic!("Expected Enclose, got {:?}", parse[0]);
            };
        }
    }

    #[test]
    fn a_fcolorbox_builder() {
        // should not fail
        to_build(r"\fcolorbox{blue}{yellow}{a b}", ParserConfig::default());
        to_build(r"\fcolorbox{blue}{yellow}{a b}^2", ParserConfig::default());
        to_build(r"\fcolorbox{blue}{yellow} x", ParserConfig::default());

        // Skipping class checks for now.
    }

    #[test]
    fn a_strike_through_parser() {
        // should not fail
        to_parse(r"\cancel{x}", ParserConfig::default());
        to_parse(r"\cancel{x^2}", ParserConfig::default());
        to_parse(r"\cancel{x}^2", ParserConfig::default());
        to_parse(r"\cancel x", ParserConfig::default());

        // should produce enclose
        {
            let parse = parse_tree(r"\cancel x", ParserConfig::default()).unwrap();
            let ParseNode::Enclose(_) = &parse[0] else {
                panic!("Expected Enclose, got {:?}", parse[0]);
            };
        }

        // should be grouped more tightly than supsubs
        {
            let parse = parse_tree(r"\cancel x^2", ParserConfig::default()).unwrap();
            let ParseNode::SupSub(_) = &parse[0] else {
                panic!("Expected SupSub, got {:?}", parse[0]);
            };
        }
    }

    #[test]
    fn a_strike_through_builder() {
        // should not fail
        to_build(r"\cancel{x}", ParserConfig::default());
        to_build(r"\cancel{x}^2", ParserConfig::default());
        to_build(r"\cancel{x}_2", ParserConfig::default());
        to_build(r"\cancel{x}_2^2", ParserConfig::default());
        to_build(r"\sout{x}", ParserConfig::default());
        to_build(r"\sout{x}^2", ParserConfig::default());
        to_build(r"\sout{x}_2", ParserConfig::default());
        to_build(r"\sout{x}_2^2", ParserConfig::default());

        // Skipping class checks for now.
    }

    #[test]
    fn a_actuarial_angle_parser() {
        // should not fail in math mode
        to_parse(r"a_{\angl{n}}", ParserConfig::default());
        // should fail in text mode
        let mut text_mode_conf = ParserConfig::default();
        text_mode_conf.display_mode = false;
        to_not_parse(r"\text{a_{\angl{n}}}", text_mode_conf);
    }

    #[test]
    fn a_actuarial_angle_builder() {
        // should not fail
        to_build(r"a_{\angl{n}}", ParserConfig::default());
        to_build(r"a_{\angl{n}i}", ParserConfig::default());
        to_build(r"a_\angln", ParserConfig::default());
        to_build(r"a_\angln", ParserConfig::default());
    }

    #[test]
    fn phase() {
        // should fail in text mode
        let mut text_mode_conf = ParserConfig::default();
        text_mode_conf.display_mode = false;
        to_not_parse(r"\text{\phase{-78.2^\circ}}", text_mode_conf);
        // should not fail in math mode
        to_build(r"\phase{-78.2^\circ}", ParserConfig::default());
    }

    #[test]
    fn a_phantom_parser() {
        // should not fail
        to_parse(r"\phantom{x}", ParserConfig::default());
        to_parse(r"\phantom{x^2}", ParserConfig::default());
        to_parse(r"\phantom{x}^2", ParserConfig::default());
        to_parse(r"\phantom x", ParserConfig::default());
        to_parse(r"\hphantom{x}", ParserConfig::default());
        to_parse(r"\hphantom{x^2}", ParserConfig::default());
        to_parse(r"\hphantom{x}^2", ParserConfig::default());
        to_parse(r"\hphantom x", ParserConfig::default());

        // should build a phantom node
        {
            let parse = parse_tree(r"\phantom{x}", ParserConfig::default()).unwrap();
            let ParseNode::Phantom(_) = &parse[0] else {
                panic!("Expected Phantom, got {:?}", parse[0]);
            };
        }
    }

    #[test]
    fn a_phantom_builder() {
        // should not fail
        to_build(r"\phantom{x}", ParserConfig::default());
        to_build(r"\phantom{x^2}", ParserConfig::default());
        to_build(r"\phantom{x}^2", ParserConfig::default());
        to_build(r"\phantom x", ParserConfig::default());
        to_build(r"\mathstrut", ParserConfig::default());

        to_build(r"\hphantom{x}", ParserConfig::default());
        to_build(r"\hphantom{x^2}", ParserConfig::default());
        to_build(r"\hphantom{x}^2", ParserConfig::default());
        to_build(r"\hphantom x", ParserConfig::default());

        // Skipping style checks for now.
    }
}
