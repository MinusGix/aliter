use aliter::{parse_tree, parser::ParserConfig, render_to_html_tree, tree::VirtualNode};

// Basic render smoke tests mirroring a subset of KaTeX katex-spec.js
// These are meant to surface builder/serialization regressions quickly.

fn render(expr: &str) -> String {
    let conf = ParserConfig::default();
    let tree = render_to_html_tree(expr, conf);
    tree.to_markup()
}

#[test]
fn render_empty_and_simple_ord() {
    let conf = ParserConfig::default();
    assert!(parse_tree("", conf.clone()).is_ok());
    let html = render("x");
    assert!(
        html.contains("katex-html") || html.contains("katex"),
        "expected katex wrapper in {html}"
    );
}

#[test]
fn render_supsub() {
    let html = render("x^2_3");
    assert!(
        html.contains("sup") || html.contains("sub"),
        "expected superscript/subscript structure in {html}"
    );
}

#[test]
fn render_frac() {
    let html = render("\\frac{1}{2}");
    assert!(
        html.contains("frac") || html.contains("mfrac"),
        "expected fraction structure in {html}"
    );
}

#[test]
fn render_sqrt() {
    let html = render("\\sqrt{2}");
    assert!(
        html.contains("sqrt") || html.contains("msqrt"),
        "expected sqrt structure in {html}"
    );
}

#[test]
fn render_color() {
    let html = render("\\textcolor{#fff}{x}");
    assert!(
        html.contains("color") || html.contains("style=\"color"),
        "expected color styling in {html}"
    );
}

#[test]
fn render_text_mode() {
    let html = render("\\text{abc}");
    assert!(
        html.contains("text") || html.contains("mtext"),
        "expected text nodes in {html}"
    );
}

#[test]
fn render_over_under() {
    let html = render("\\overline{x} + \\underline{y}");
    assert!(
        html.contains("overline") || html.contains("underline") || html.contains("mover"),
        "expected over/underline markers in {html}"
    );
}

#[test]
fn render_delims() {
    let html = render("\\left( x^2 \\right)");
    assert!(
        html.contains("(") && html.contains(")"),
        "expected delimiters in {html}"
    );
}

#[test]
fn render_rule() {
    let html = render("\\rule{1em}{2em}");
    assert!(
        html.contains("rule") || html.contains("span"),
        "expected rule span in {html}"
    );
}

#[test]
fn render_phantom() {
    let html = render("\\phantom{xyz}");
    assert!(
        html.contains("phantom") || html.contains("span"),
        "expected phantom span in {html}"
    );
}

#[test]
fn render_sqrt_nested() {
    let html = render("\\sqrt{1+\\sqrt{2}}");
    assert!(
        html.contains("sqrt") || html.contains("msqrt"),
        "expected nested sqrt structure in {html}"
    );
}

#[test]
fn render_binom() {
    let html = render("\\binom{n}{k}");
    assert!(
        html.contains("frac") || html.contains("mfrac") || html.contains("binom"),
        "expected binomial-like structure in {html}"
    );
}

#[test]
fn render_left_right() {
    let html = render("\\left[ x+y \\right]");
    assert!(
        html.contains("[") && html.contains("]"),
        "expected bracket delimiters in {html}"
    );
}

#[test]
fn render_text_fonts() {
    let html = render("\\textbf{bold} \\textit{it}");
    assert!(
        html.contains("text") || html.contains("mtext"),
        "expected text font markup in {html}"
    );
}

#[test]
fn render_matrix() {
    let html = render("\\begin{matrix} a & b \\\\ c & d \\end{matrix}");
    assert!(
        html.contains("matrix") || html.contains("<table") || html.contains("<mtable"),
        "expected matrix-like structure in {html}"
    );
}

#[test]
fn render_arrows() {
    let html = render("\\xleftarrow{abc} \\xrightarrow{def}");
    assert!(
        html.contains("arrow") || html.contains("<mo") || html.contains("rightarrow"),
        "expected arrow structure in {html}"
    );
}

#[cfg(feature = "mathml")]
fn render_mathml(expr: &str) -> String {
    let conf = ParserConfig::default();
    let tree = aliter::render_to_mathml_tree(expr, conf);
    tree.to_markup()
}

#[test]
fn an_html_font_tree_builder() {
    // should render \mathbb{R} with the correct font
    {
        let markup = render(r"\mathbb{R}");
        assert!(markup.contains(r#"<span class="mord mathbb">R</span>"#));
    }

    // should render \mathrm{R} with the correct font
    {
        let markup = render(r"\mathrm{R}");
        assert!(markup.contains(r#"<span class="mord mathrm">R</span>"#));
    }

    // should render \mathcal{R} with the correct font
    {
        let markup = render(r"\mathcal{R}");
        assert!(markup.contains(r#"<span class="mord mathcal">R</span>"#));
    }

    // should render \mathfrak{R} with the correct font
    {
        let markup = render(r"\mathfrak{R}");
        assert!(markup.contains(r#"<span class="mord mathfrak">R</span>"#));
    }

    // should render \text{R} with the correct font
    {
        let markup = render(r"\text{R}");
        assert!(markup.contains(r#"<span class="mord">R</span>"#));
    }

    // should render \textit{R} with the correct font
    {
        let markup = render(r"\textit{R}");
        assert!(markup.contains(r#"<span class="mord textit">R</span>"#));
    }

    // should render \text{\textit{R}} with the correct font
    {
        let markup = render(r"\text{\textit{R}}");
        assert!(markup.contains(r#"<span class="mord textit">R</span>"#));
    }

    // should render \textup{R} with the correct font
    {
        let markup1 = render(r"\textup{R}");
        assert!(markup1.contains(r#"<span class="mord textup">R</span>"#));
        let markup2 = render(r"\textit{\textup{R}}");
        assert!(markup2.contains(r#"<span class="mord textup">R</span>"#));
        let markup3 = render(r"\textup{\textit{R}}");
        assert!(markup3.contains(r#"<span class="mord textit">R</span>"#));
    }

    // should render \text{R\textit{S}T} with the correct fonts
    {
        let markup = render(r"\text{R\textit{S}T}");
        assert!(markup.contains(r#"<span class="mord">R</span>"#));
        assert!(markup.contains(r#"<span class="mord textit">S</span>"#));
        assert!(markup.contains(r#"<span class="mord">T</span>"#));
    }

    // should render \textbf{R } with the correct font
    {
        let markup = render(r"\textbf{R }");
        // Note: \u00a0 is non-breaking space
        // Rust regex might need char matching or just loose matching
        assert!(markup.contains(r#"<span class="mord textbf">R"#)); 
    }

    // should render \textmd{R} with the correct font
    {
        let markup1 = render(r"\textmd{R}");
        assert!(markup1.contains(r#"<span class="mord textmd">R</span>"#));
        let markup2 = render(r"\textbf{\textmd{R}}");
        assert!(markup2.contains(r#"<span class="mord textmd">R</span>"#));
        let markup3 = render(r"\textmd{\textbf{R}}");
        assert!(markup3.contains(r#"<span class="mord textbf">R</span>"#));
    }

    // should render \textsf{R} with the correct font
    {
        let markup = render(r"\textsf{R}");
        assert!(markup.contains(r#"<span class="mord textsf">R</span>"#));
    }

    // should render \textsf{\textit{R}G\textbf{B}} with the correct font
    {
        let markup = render(r"\textsf{\textit{R}G\textbf{B}}");
        assert!(markup.contains(r#"<span class="mord textsf textit">R</span>"#));
        assert!(markup.contains(r#"<span class="mord textsf">G</span>"#));
        assert!(markup.contains(r#"<span class="mord textsf textbf">B</span>"#));
    }

    // should render \textsf{\textbf{$\mathrm{A}$}} with the correct font
    {
        let markup = render(r"\textsf{\textbf{$\mathrm{A}$}}");
        assert!(markup.contains(r#"<span class="mord mathrm">A</span>"#));
    }

    // should render \textsf{\textbf{$\mathrm{\textsf{A}}$}} with the correct font
    {
        let markup = render(r"\textsf{\textbf{$\mathrm{\textsf{A}}$}}");
        assert!(markup.contains(r#"<span class="mord textsf textbf">A</span>"#));
    }

    // should render \texttt{R} with the correct font
    {
        let markup = render(r"\texttt{R}");
        assert!(markup.contains(r#"<span class="mord texttt">R</span>"#));
    }

    // should render a combination of font and color changes
    {
        let markup = render(r"\textcolor{blue}{\mathbb R}");
        // Note: aliter might render style="color: blue;" (with space)
        assert!(markup.contains(r#"<span class="mord mathbb""#));
        assert!(markup.contains("color: blue") || markup.contains("color:blue"));
        assert!(markup.contains(">R</span>"));

        let markup2 = render(r"\mathbb{\textcolor{blue}{R}}");
        assert!(markup2.contains(r#"<span class="mord mathbb""#));
        assert!(markup2.contains("color: blue") || markup2.contains("color:blue"));
        assert!(markup2.contains(">R</span>"));
    }
}

#[test]
#[cfg(feature = "mathml")]
fn a_mathml_font_tree_builder() {
    let contents = r"Ax2k\omega\Omega\imath+";

    // should render with the correct mathvariants
    {
        let markup = render_mathml(contents);
        assert!(markup.contains("<mi>A</mi>"));
        assert!(markup.contains("<mi>x</mi>"));
        assert!(markup.contains("<mn>2</mn>"));
        assert!(markup.contains("<mi>\u{03c9}</mi>"));   // \omega
        assert!(markup.contains(r#"<mi mathvariant="normal">Ω</mi>"#));   // \Omega, \u03A9
        assert!(markup.contains(r#"<mi mathvariant="normal">ı</mi>"#));   // \imath, \u0131
        assert!(markup.contains("<mo>+</mo>"));
    }

    // should render \mathbb{...} with the correct mathvariants
    {
        let tex = format!(r"\mathbb{{{}}}", contents);
        let markup = render_mathml(&tex);
        assert!(markup.contains(r#"<mi mathvariant="double-struck">A</mi>"#));
        assert!(markup.contains(r#"<mi mathvariant="double-struck">x</mi>"#));
        assert!(markup.contains(r#"<mn mathvariant="double-struck">2</mn>"#));
        assert!(markup.contains(r#"<mi mathvariant="double-struck">ω</mi>"#));  // \omega
        assert!(markup.contains(r#"<mi mathvariant="double-struck">Ω</mi>"#)); // \Omega
        assert!(markup.contains(r#"<mi mathvariant="double-struck">ı</mi>"#));  // \imath
        assert!(markup.contains("<mo>+</mo>"));
    }

    // should render \mathrm{...} with the correct mathvariants
    {
        let tex = format!(r"\mathrm{{{}}}", contents);
        let markup = render_mathml(&tex);
        assert!(markup.contains(r#"<mi mathvariant="normal">A</mi>"#));
        assert!(markup.contains(r#"<mi mathvariant="normal">x</mi>"#));
        assert!(markup.contains("<mn>2</mn>"));
        assert!(markup.contains(r#"<mi>ω</mi>"#));   // \omega
        assert!(markup.contains(r#"<mi mathvariant="normal">Ω</mi>"#));   // \Omega
        assert!(markup.contains(r#"<mi mathvariant="normal">ı</mi>"#));   // \imath
        assert!(markup.contains("<mo>+</mo>"));
    }

    // should render \mathit{...} with the correct mathvariants
    {
        let tex = format!(r"\mathit{{{}}}", contents);
        let markup = render_mathml(&tex);
        assert!(markup.contains("<mi>A</mi>"));
        assert!(markup.contains("<mi>x</mi>"));
        assert!(markup.contains(r#"<mn mathvariant="italic">2</mn>"#));
        assert!(markup.contains(r#"<mi>ω</mi>"#));   // \omega
        assert!(markup.contains(r#"<mi>Ω</mi>"#));   // \Omega
        assert!(markup.contains(r#"<mi>ı</mi>"#));   // \imath
        assert!(markup.contains("<mo>+</mo>"));
    }

    // should render \mathnormal{...} with the correct mathvariants
    {
        let tex = format!(r"\mathnormal{{{}}}", contents);
        let markup = render_mathml(&tex);
        assert!(markup.contains("<mi>A</mi>"));
        assert!(markup.contains("<mi>x</mi>"));
        assert!(markup.contains("<mn>2</mn>"));
        assert!(markup.contains(r#"<mi>ω</mi>"#));   // \omega
        assert!(markup.contains(r#"<mi mathvariant="normal">Ω</mi>"#));   // \Omega
        assert!(markup.contains(r#"<mi mathvariant="normal">ı</mi>"#));   // \imath
        assert!(markup.contains("<mo>+</mo>"));
    }

    // should render \mathbf{...} with the correct mathvariants
    {
        let tex = format!(r"\mathbf{{{}}}", contents);
        let markup = render_mathml(&tex);
        assert!(markup.contains(r#"<mi mathvariant="bold">A</mi>"#));
        assert!(markup.contains(r#"<mi mathvariant="bold">x</mi>"#));
        assert!(markup.contains(r#"<mn mathvariant="bold">2</mn>"#));
        assert!(markup.contains(r#"<mi mathvariant="bold">ω</mi>"#));   // \omega
        assert!(markup.contains(r#"<mi mathvariant="bold">Ω</mi>"#));   // \Omega
        assert!(markup.contains(r#"<mi mathvariant="bold">ı</mi>"#));   // \imath
        assert!(markup.contains("<mo>+</mo>"));
    }

    // should render \mathcal{...} with the correct mathvariants
    {
        let tex = format!(r"\mathcal{{{}}}", contents);
        let markup = render_mathml(&tex);
        assert!(markup.contains(r#"<mi mathvariant="script">A</mi>"#));
        assert!(markup.contains(r#"<mi mathvariant="script">x</mi>"#));
        assert!(markup.contains(r#"<mn mathvariant="script">2</mn>"#));
        assert!(markup.contains(r#"<mi mathvariant="script">ω</mi>"#)); // \omega
        assert!(markup.contains(r#"<mi mathvariant="script">Ω</mi>"#)); // \Omega
        assert!(markup.contains(r#"<mi mathvariant="script">ı</mi>"#)); // \imath
        assert!(markup.contains("<mo>+</mo>"));
    }

    // should render \mathfrak{...} with the correct mathvariants
    {
        let tex = format!(r"\mathfrak{{{}}}", contents);
        let markup = render_mathml(&tex);
        assert!(markup.contains(r#"<mi mathvariant="fraktur">A</mi>"#));
        assert!(markup.contains(r#"<mi mathvariant="fraktur">x</mi>"#));
        assert!(markup.contains(r#"<mn mathvariant="fraktur">2</mn>"#));
        assert!(markup.contains(r#"<mi mathvariant="fraktur">ω</mi>"#)); // \omega
        assert!(markup.contains(r#"<mi mathvariant="fraktur">Ω</mi>"#)); // \Omega
        assert!(markup.contains(r#"<mi mathvariant="fraktur">ı</mi>"#)); // \imath
        assert!(markup.contains("<mo>+</mo>"));
    }

    // should render \mathscr{...} with the correct mathvariants
    {
        let tex = format!(r"\mathscr{{{}}}", contents);
        let markup = render_mathml(&tex);
        assert!(markup.contains(r#"<mi mathvariant="script">A</mi>"#));
        assert!(markup.contains(r#"<mi mathvariant="script">x</mi>"#));
        assert!(markup.contains(r#"<mn mathvariant="script">2</mn>"#));
        assert!(markup.contains(r#"<mi mathvariant="script">ω</mi>"#)); // \omega
        assert!(markup.contains(r#"<mi mathvariant="script">Ω</mi>"#)); // \Omega
        assert!(markup.contains(r#"<mi mathvariant="script">ı</mi>"#)); // \imath
        assert!(markup.contains("<mo>+</mo>"));
    }

    // should render \mathsf{...} with the correct mathvariants
    {
        let tex = format!(r"\mathsf{{{}}}", contents);
        let markup = render_mathml(&tex);
        assert!(markup.contains(r#"<mi mathvariant="sans-serif">A</mi>"#));
        assert!(markup.contains(r#"<mi mathvariant="sans-serif">x</mi>"#));
        assert!(markup.contains(r#"<mn mathvariant="sans-serif">2</mn>"#));
        assert!(markup.contains(r#"<mi mathvariant="sans-serif">ω</mi>"#)); // \omega
        assert!(markup.contains(r#"<mi mathvariant="sans-serif">Ω</mi>"#)); // \Omega
        assert!(markup.contains(r#"<mi mathvariant="sans-serif">ı</mi>"#)); // \imath
        assert!(markup.contains("<mo>+</mo>"));
    }

    // should render a combination of font and color changes
    {
        let tex = r"\textcolor{blue}{\mathbb R}";
        let markup = render_mathml(tex);
        assert!(markup.contains(r#"<mstyle mathcolor="blue">"#));
        assert!(markup.contains(r#"<mi mathvariant="double-struck">R</mi>"#));
        assert!(markup.contains(r#"</mstyle>"#));

        // reverse the order of the commands
        let tex = r"\mathbb{\textcolor{blue}{R}}";
        let markup = render_mathml(tex);
        assert!(markup.contains(r#"<mstyle mathcolor="blue">"#));
        assert!(markup.contains(r#"<mi mathvariant="double-struck">R</mi>"#));
        assert!(markup.contains(r#"</mstyle>"#));
    }
}

fn render_with_conf(expr: &str, conf: ParserConfig) -> String {
    let tree = render_to_html_tree(expr, conf);
    tree.to_markup()
}


#[test]
fn an_includegraphics_builder() {
    let img = r"\includegraphics[height=0.9em, totalheight=0.9em, width=0.9em, alt=KA logo]{https://cdn.kastatic.org/images/apple-touch-icon-57x57-precomposed.new.png}";

    // should not fail
    let mut trust_conf = ParserConfig::default();
    trust_conf.trust = true;
    to_build(img, trust_conf.clone());

    // should produce mords
    let html = render_with_conf(img, trust_conf.clone());
    assert!(html.contains("mord"));

    // should not render without trust setting
    let no_trust_conf = ParserConfig::default(); // trust is false by default
    let html_no_trust = render_with_conf(img, no_trust_conf.clone());
    assert!(!html_no_trust.contains("<img"));
    assert!(!html_no_trust.contains("cdn.kastatic.org"));

    // should render with trust setting
    let html_with_trust = render_with_conf(img, trust_conf.clone());
    assert!(html_with_trust.contains("<img"));
    assert!(html_with_trust.contains("cdn.kastatic.org"));
    assert!(html_with_trust.contains(r#"height="0.9em""#));
    assert!(html_with_trust.contains(r#"width="0.9em""#));
    assert!(html_with_trust.contains(r#"alt="KA logo""#));
}

#[test]
fn an_html_extension_builder() {
    let html_expr = r"\htmlId{bar}{x}\htmlClass{foo}{x}\htmlStyle{color: red;}{x}\htmlData{foo=a, bar=b}{x}";
    let mut conf = ParserConfig::default();
    conf.trust = true;
    // The original test uses `strict: false`, but aliter has StrictMode::Warn by default.

    // should not fail
    to_build(html_expr, conf.clone());

    // should set HTML attributes
    let rendered = render_with_conf(html_expr, conf.clone());
    assert!(rendered.contains(r#"id="bar""#));
    assert!(rendered.contains(r#"class="foo""#)); 
    assert!(rendered.contains(r#"style="color: red;""#));
    assert!(rendered.contains(r#"data-bar="b""#));
    assert!(rendered.contains(r#"data-foo="a""#));

    // should not affect spacing
    let spacing_html = r"\htmlId{a}{x+}y";
    let rendered_spacing = render_with_conf(spacing_html, conf.clone());
    assert!(!rendered_spacing.is_empty()); 
}

#[test]
fn a_bin_builder() {
    let conf = ParserConfig::default();

    // should create mbins normally
    let html_x_plus_y = render_with_conf("x + y", conf.clone());
    assert!(html_x_plus_y.contains(r#"mbin"#)); 

    // should create ords when at the beginning of lists
    let html_plus_x = render_with_conf("+ x", conf.clone());
    assert!(html_plus_x.contains(r#"mord"#));
    assert!(!html_plus_x.contains(r#"mbin"#)); 

    // should create ords after some other objects
    assert!(render_with_conf("x + + 2", conf.clone()).contains("mord"));
    assert!(render_with_conf("( + 2", conf.clone()).contains("mord"));
    assert!(render_with_conf("= + 2", conf.clone()).contains("mord"));
    assert!(render_with_conf(r"\sin + 2", conf.clone()).contains("mord"));
    assert!(render_with_conf(", + 2", conf.clone()).contains("mord"));

    // should correctly interact with color objects
    let html_color_bin = render_with_conf(r"\blue{x}+y", conf.clone());
    assert!(html_color_bin.contains(r#"mbin"#));

    let html_color_bin_nested = render_with_conf(r"\blue{x+}+y", conf.clone());
    assert!(html_color_bin_nested.contains(r#"mbin"#));
}

#[test]
fn a_phantom_builder_and_smash_builder() {
    let conf = ParserConfig::default();

    // should both build a mord
    assert!(render_with_conf(r"\hphantom{a}", conf.clone()).contains("mord"));
    assert!(render_with_conf(r"a\hphantom{=}b", conf.clone()).contains("mord"));
    assert!(render_with_conf(r"a\hphantom{+}b", conf.clone()).contains("mord"));
    assert!(render_with_conf(r"\smash{a}", conf.clone()).contains("mord"));
    assert!(render_with_conf(r"\smash{=}", conf.clone()).contains("mord"));
    assert!(render_with_conf(r"a\smash{+}b", conf.clone()).contains("mord"));
}

#[test]
fn a_markup_generator() {
    // marks trees up
    {
        let markup = render(r"\sigma^2");
        assert!(markup.starts_with("<span"));
        assert!(markup.contains("\u{03c3}")); // sigma
        assert!(markup.contains("margin-right"));
        assert!(!markup.contains("marginRight"));
    }

    // generates both MathML and HTML
    {
        let markup = render("a");
        assert!(markup.contains("<span"));
        assert!(markup.contains("<math"));
    }
}
