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
        style::TEXT_STYLE,
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
}
