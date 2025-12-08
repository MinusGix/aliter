use std::{borrow::Cow, sync::Arc};

use once_cell::sync::Lazy;

use crate::{
    expander::{MacroExpander, Mode},
    font_metrics_data, functions,
    lexer::Token,
    macr::{MacroExpansion, MacroReplace, MacroVal, Macros},
    parser::ParseError,
    symbols::{self, Atom, Group},
    unit::make_em,
    util::{char_code_for, find_assoc_data},
};

fn f(
    cb: impl for<'a, 'f> Fn(&mut MacroExpander<'a, 'f>) -> Result<MacroVal<'a, 'static>, ParseError>
        + Send
        + Sync
        + 'static,
) -> Arc<MacroReplace> {
    Arc::new(MacroReplace::Func(Box::new(cb)))
}

fn text(text: &'static str) -> Arc<MacroReplace> {
    Arc::new(MacroReplace::Text(text.to_owned()))
}

pub static BUILTIN_MACROS: Lazy<Macros> = Lazy::new(|| {
    let mut macros = Macros::default();

    // macro tools

    macros.insert_back_macro(
        "\\noexpand",
        f(|exp| {
            // The expansion is the token itself; but that token is interpreted as if its meaning
            // were `\relax` if it is a control sequence that would ordinarily be expanded by TeX's
            // expansion rules.
            let mut t = exp.pop_token()?;
            if exp.is_expandable(&t.content) {
                t.no_expand = true;
                t.treat_as_relax = true;
            }

            Ok(MacroExpansion::new(vec![t], 0).into())
        }),
    );

    macros.insert_back_macro(
        "\\expandafter",
        f(|exp| {
            // TeX first reads the token that comes immediately after \expandafter,
            // without expanding it; let’s call this token t. Then TeX reads the
            // token that comes after t (and possibly more tokens, if that token
            // has an argument), replacing it by its expansion. Finally TeX puts
            // t back in front of that expansion.
            let t = exp.pop_token()?;
            // expand only an expandable token
            exp.expand_once(true)?;

            Ok(MacroExpansion::new(vec![t], 0).into())
        }),
    );

    // LaTeX's `\@firstoftwo{#1}{#2}` expand to `#1`, skpping `#2`
    // TeX source: \long\def\@firstoftwo#1#2{#1}
    macros.insert_back_macro(
        "\\@firstoftwo",
        f(|exp| {
            let args = exp.consume_args_n::<2>()?;
            let tokens = args.into_iter().nth(0).unwrap();
            Ok(MacroExpansion::new(tokens, 0).into())
        }),
    );

    // LaTeX's \@secondoftwo{#1}{#2} expands to #2, skipping #1
    // TeX source: \long\def\@secondoftwo#1#2{#2}
    macros.insert_back_macro(
        "\\@secondoftwo",
        f(|exp| {
            let args = exp.consume_args_n::<2>()?;
            let tokens = args.into_iter().nth(1).unwrap();
            Ok(MacroExpansion::new(tokens, 0).into())
        }),
    );

    // LaTeX's \@ifnextchar{#1}{#2}{#3} looks ahead to the next (unexpanded)
    // symbol that isn't a space, consuming any spaces but not consuming the
    // first nonspace character.  If that nonspace character matches #1, then
    // the macro expands to #2; otherwise, it expands to #3.
    macros.insert_back_macro(
        "\\@ifnextchar",
        f(|exp| {
            // symbol, if, else
            let args = exp.consume_args_n::<3>()?;
            exp.consume_spaces()?;
            let next_token = exp.future()?;
            let idx = if args[0].len() == 1 && args[0][0].content == next_token.content {
                1
            } else {
                2
            };
            let tokens = args.into_iter().nth(idx).unwrap();

            Ok(MacroExpansion::new(tokens, 0).into())
        }),
    );

    // LaTeX's \@ifstar{#1}{#2} looks ahead to the next (unexpanded) symbol.
    // If it is `*`, then it consumes the symbol, and the macro expands to #1;
    // otherwise, the macro expands to #2 (without consuming the symbol).
    // TeX source: \def\@ifstar#1{\@ifnextchar *{\@firstoftwo{#1}}}
    macros.insert_back_macro(
        "\\@ifstar",
        Arc::new(MacroReplace::Text(
            "\\@ifnextchar *{\\@firstoftwo{#1}}".to_string(),
        )),
    );

    // LaTeX's \TextOrMath{#1}{#2} expands to #1 in text mode, #2 in math mode
    macros.insert_back_macro(
        "\\TextOrMath",
        f(|exp| {
            let args = exp.consume_args_n::<2>()?;
            let idx = match exp.mode {
                Mode::Text => 0,
                Mode::Math => 1,
            };
            let tokens = args.into_iter().nth(idx).unwrap();
            Ok(MacroExpansion::new(tokens, 0).into())
        }),
    );

    // TeX \char makes a literal character (catcode 12) using the following forms:
    // (see The TeXBook, p. 43)
    //   \char123  -- decimal
    //   \char'123 -- octal
    //   \char"123 -- hex
    //   \char`x   -- character that can be written (i.e. isn't active)
    //   \char`\x  -- character that cannot be written (e.g. %)
    // These all refer to characters from the font, so we turn them into special
    // calls to a function \@char dealt with in the Parser.
    macros.insert_back_macro(
        "\\char",
        f(|exp| {
            let token = exp.pop_token()?;
            let mut number: Option<char> = None;
            let (base, token) = match token.content.as_ref() {
                "'" => (Some(8), exp.pop_token()?),
                "\"" => (Some(16), exp.pop_token()?),
                "`" => {
                    let token = exp.pop_token()?;
                    if token.content.starts_with('\\') {
                        let token_char = token.content.chars().nth(1).unwrap();
                        number = Some(token_char);
                    } else if token.is_eof() {
                        return Err(ParseError::CharMissingArgument);
                    } else {
                        let token_char = token.content.chars().nth(0).unwrap();
                        number = Some(token_char);
                    }

                    (None, token)
                }
                _ => (Some(10), token),
            };

            if let Some(base) = base {
                // Parse a number in the given base, starting with the first token
                debug_assert_eq!(token.content.len(), 1);
                let number = token.content.chars().nth(0).unwrap();
                let mut number =
                    ch_to_digit(number).ok_or_else(|| ParseError::CharInvalidBaseDigit)?;

                loop {
                    let token = exp.future()?;
                    if token.is_eof() || token.content.len() != 1 {
                        break;
                    }

                    let digit = token.content.chars().nth(0).unwrap();
                    let Some(digit) = ch_to_digit(digit) else {
                        break;
                    };

                    number *= base;
                    number += digit;
                    exp.pop_token()?;
                }

                Ok(MacroVal::Text(Cow::Owned(format!("\\@char{{{}}}", number))))
            } else if let Some(number) = number {
                let number = char_code_for(number);
                Ok(MacroVal::Text(Cow::Owned(format!("\\@char{{{}}}", number))))
            } else {
                Ok(MacroVal::Text(Cow::Borrowed("\\@char{}")))
            }
        }),
    );

    // \newcommand{\macro}[args]{definition}
    // \renewcommand{\macro}[args]{definition}
    // TODO: Optional arguments: \newcommand{\macro}[args][default]{definition}
    fn new_command<'a, 'f, const EXISTS_OK: bool, const NON_EXISTS_OK: bool>(
        exp: &mut MacroExpander<'a, 'f>,
    ) -> Result<MacroVal<'a, 'static>, ParseError> {
        let arg = exp.consume_arg()?.tokens;
        if arg.len() != 1 {
            return Err(ParseError::NewCommandFirstArgMustBeName);
        }

        let name = &arg[0].content;

        let exists = exp.is_defined(name);
        if exists && !EXISTS_OK {
            return Err(ParseError::NewCommandAttemptingToRedefine(name.to_string()));
        }
        if !exists && !NON_EXISTS_OK {
            return Err(ParseError::NewCommandAttemptingToDefine(name.to_string()));
        }

        let mut num_args = 0;
        let mut arg = exp.consume_arg()?.tokens;
        if arg.len() == 1 && arg[0].content == "[" {
            let mut arg_text = String::new();
            let mut token = exp.expand_next_token()?;
            while token.content != "]" && !token.is_eof() {
                // TODO: should properly expand arg, e.g., ignore {}s
                arg_text.push_str(&token.content);
                token = exp.expand_next_token()?;
            }

            // TODO: don't unwrap
            num_args = arg_text.parse().unwrap();
            arg = exp.consume_arg()?.tokens;
        }

        // We use into_owned because we're setting it as a macro which is static
        let arg = arg.into_iter().map(|x| x.into_owned()).collect();
        // TODO: does name include the backslash?
        exp.macros.set_back_macro(
            name.to_string(),
            Some(Arc::new(MacroReplace::Expansion(MacroExpansion::new(
                arg, num_args,
            )))),
        );

        Ok(MacroVal::empty_text())
    }

    macros.insert_back_macro("\\newcommand", f(new_command::<false, true>));
    macros.insert_back_macro("\\renewcommand", f(new_command::<true, false>));
    macros.insert_back_macro("\\providecommand", f(new_command::<true, true>));

    // TODO: ensure these aren't used anywhere because they might not be wanted?
    // terminal (console) tools
    macros.insert_back_macro(
        "\\message",
        f(|exp| {
            let mut arg = exp.consume_arg()?.tokens;
            arg.reverse();
            let arg = arg
                .into_iter()
                .map(|x| x.content)
                .collect::<Vec<_>>()
                .join("");
            println!("{}", arg);
            Ok(MacroVal::empty_text())
        }),
    );
    macros.insert_back_macro(
        "\\errmessage",
        f(|exp| {
            let arg = exp.consume_arg()?.tokens;
            let arg = arg
                .into_iter()
                .map(|x| x.content)
                .collect::<Vec<_>>()
                .join("");
            eprintln!("{}", arg);
            Ok(MacroVal::empty_text())
        }),
    );
    macros.insert_back_macro(
        "\\show",
        f(|exp| {
            let tok = exp.pop_token()?;
            let name = &tok.content;

            let macr = exp.macros.get_back_macro(name);
            let func = &*functions::FUNCTIONS;
            let func = func.get(name);
            let sym_math = symbols::SYMBOLS.get(Mode::Math, name);
            let sym_text = symbols::SYMBOLS.get(Mode::Text, name);
            println!("{tok:?} {macr:?} {func:?} {sym_math:?} {sym_text:?}");
            Ok(MacroVal::empty_text())
        }),
    );

    // Grouping

    // \let\bgroup={ \let\egroup=}
    macros.insert_back_macro("\\bgroup", text("{"));
    macros.insert_back_macro("\\egroup", text("}"));

    // Symbols from latex.ltx:
    // \def~{\nobreakspace{}}
    // \def\lq{`}
    // \def\rq{'}
    // \def \aa {\r a}
    // \def \AA {\r A}
    macros.insert_back_macro("~", text("\\nobreakspace"));
    macros.insert_back_macro("\\lq", text("`"));
    macros.insert_back_macro("\\rq", text("'"));
    macros.insert_back_macro("\\aa", text("\\r a"));
    macros.insert_back_macro("\\AA", text("\\r A"));

    // Copyright (C) and registered (R) symbols. Use raw symbol in MathML.
    // \DeclareTextCommandDefault{\textcopyright}{\textcircled{c}}
    // \DeclareTextCommandDefault{\textregistered}{\textcircled{%
    //      \check@mathfonts\fontsize\sf@size\z@\math@fontsfalse\selectfont R}}
    // \DeclareRobustCommand{\copyright}{%
    //    \ifmmode{\nfss@text{\textcopyright}}\else\textcopyright\fi}
    macros.insert_back_macro(
        "\\textcopyright",
        text("\\html@mathml{\\textcircled{c}}{\\char`©}"),
    );
    macros.insert_back_macro(
        "\\copyright",
        text("\\TextOrMath{\\textcopyright}{\\text{\\textcopyright}}"),
    );
    macros.insert_back_macro(
        "\\textregistered",
        text("\\html@mathml{\\textcircled{\\scriptsize R}}{\\char`®}"),
    );

    // Characters omitted from Unicode range 1D400–1D7FF
    macros.insert_back_macro("\u{212C}", text("\\mathscr{B}")); // script
    macros.insert_back_macro("\u{2130}", text("\\mathscr{E}"));
    macros.insert_back_macro("\u{2131}", text("\\mathscr{F}"));
    macros.insert_back_macro("\u{210B}", text("\\mathscr{H}"));
    macros.insert_back_macro("\u{2110}", text("\\mathscr{I}"));
    macros.insert_back_macro("\u{2112}", text("\\mathscr{L}"));
    macros.insert_back_macro("\u{2133}", text("\\mathscr{M}"));
    macros.insert_back_macro("\u{211B}", text("\\mathscr{R}"));
    macros.insert_back_macro("\u{212D}", text("\\mathfrak{C}")); // Fraktur
    macros.insert_back_macro("\u{210C}", text("\\mathfrak{H}"));
    macros.insert_back_macro("\u{2128}", text("\\mathfrak{Z}"));

    // Define \Bbbk with a macro that works in both HTML and MathML.
    macros.insert_back_macro("\\Bbbk", text("\\Bbb{k}"));

    // Unicode middle dot
    // The KaTeX fonts do not contain U+00B7. Instead, \cdotp displays
    // the dot at U+22C5 and gives it punct spacing.
    macros.insert_back_macro("\u{00b7}", text("\\cdotp"));

    // \llap and \rlap render their contents in text mode
    macros.insert_back_macro("\\llap", text("\\mathllap{\\textrm{#1}}"));
    macros.insert_back_macro("\\rlap", text("\\mathrlap{\\textrm{#1}}"));
    macros.insert_back_macro("\\clap", text("\\mathclap{\\textrm{#1}}"));

    // \mathstrut from the TeXbook, p 360
    macros.insert_back_macro("\\mathstrut", text("\\vphantom{(}"));

    // \underbar from TeXbook p 353
    macros.insert_back_macro("\\underbar", text("\\underline{\\text{#1}}"));

    // \not is defined by base/fontmath.ltx via
    // \DeclareMathSymbol{\not}{\mathrel}{symbols}{"36}
    // It's thus treated like a \mathrel, but defined by a symbol that has zero
    // width but extends to the right.  We use \rlap to get that spacing.
    // For MathML we write U+0338 here. buildMathML.js will then do the overlay.
    macros.insert_back_macro(
        "\\not",
        text("\\html@mathml{\\mathrel{\\mathrlap\\@not}}{\\char\"338}"),
    );

    // Negated symbols from base/fontmath.ltx:
    // \def\neq{\not=} \let\ne=\neq
    // \DeclareRobustCommand
    //   \notin{\mathrel{\m@th\mathpalette\c@ncel\in}}
    // \def\c@ncel#1#2{\m@th\ooalign{$\hfil#1\mkern1mu/\hfil$\crcr$#1#2$}}
    macros.insert_back_macro(
        "\\neq",
        text("\\html@mathml{\\mathrel{\\not=}}{\\mathrel{\\char`≠}}"),
    );
    macros.insert_back_macro("\\ne", text("\\neq"));
    macros.insert_back_macro("\u{2260}", text("\\neq"));
    macros.insert_back_macro(
        "\\notin",
        text("\\html@mathml{\\mathrel{{\\in}\\mathllap{/\\mskip1mu}}}{\\mathrel{\\char`∉}}"),
    );
    macros.insert_back_macro("\u{2209}", text("\\notin"));

    // Unicode stacked relations
    macros.insert_back_macro("\u{2258}", text("\\html@mathml{\\mathrel{=\\kern{-1em}\\raisebox{0.4em}{$\\scriptsize\\frown$}}}{\\mathrel{\\char`\u{2258}}}"));
    macros.insert_back_macro(
        "\u{2259}",
        text("\\html@mathml{\\stackrel{\\tiny\\wedge}{=}}{\\mathrel{\\char`\u{2258}}}"),
    );
    macros.insert_back_macro(
        "\u{225A}",
        text("\\html@mathml{\\stackrel{\\tiny\\vee}{=}}{\\mathrel{\\char`\u{225A}}}"),
    );
    macros.insert_back_macro(
        "\u{225B}",
        text("\\html@mathml{\\stackrel{\\scriptsize\\star}{=}}{\\mathrel{\\char`\u{225B}}}"),
    );
    macros.insert_back_macro(
        "\u{225D}",
        text("\\html@mathml{\\stackrel{\\tiny\\mathrm{def}}{=}}{\\mathrel{\\char`\u{225D}}}"),
    );
    macros.insert_back_macro(
        "\u{225E}",
        text("\\html@mathml{\\stackrel{\\tiny\\mathrm{m}}{=}}{\\mathrel{\\char`\u{225E}}}"),
    );
    macros.insert_back_macro(
        "\u{225F}",
        text("\\html@mathml{\\stackrel{\\tiny?}{=}}{\\mathrel{\\char`\u{225F}}}"),
    );

    // Misc Unicode
    macros.insert_back_macro("\u{27C2}", text("\\perp"));
    macros.insert_back_macro("\u{203C}", text("\\mathclose{!\\mkern-0.8mu!}"));
    macros.insert_back_macro("\u{220C}", text("\\notni"));
    macros.insert_back_macro("\u{231C}", text("\\ulcorner"));
    macros.insert_back_macro("\u{231D}", text("\\urcorner"));
    macros.insert_back_macro("\u{231E}", text("\\llcorner"));
    macros.insert_back_macro("\u{231F}", text("\\lrcorner"));
    macros.insert_back_macro("\u{00A9}", text("\\copyright"));
    macros.insert_back_macro("\u{00AE}", text("\\textregistered"));
    macros.insert_back_macro("\u{FE0F}", text("\\textregistered"));

    // The KaTeX fonts have corners at codepoints that don't match Unicode.
    // For MathML purposes, use the Unicode code point.
    macros.insert_back_macro(
        "\\ulcorner",
        text("\\html@mathml{\\@ulcorner}{\\mathop{\\char\"231c}}"),
    );
    macros.insert_back_macro(
        "\\urcorner",
        text("\\html@mathml{\\@urcorner}{\\mathop{\\char\"231d}}"),
    );
    macros.insert_back_macro(
        "\\llcorner",
        text("\\html@mathml{\\@llcorner}{\\mathop{\\char\"231e}}"),
    );
    macros.insert_back_macro(
        "\\lrcorner",
        text("\\html@mathml{\\@lrcorner}{\\mathop{\\char\"231f}}"),
    );

    // LaTeX_2ε

    // \vdots{\vbox{\baselineskip4\p@  \lineskiplimit\z@
    // \kern6\p@\hbox{.}\hbox{.}\hbox{.}}}
    // We'll call \varvdots, which gets a glyph from symbols.js.
    // The zero-width rule gets us an equivalent to the vertical 6pt kern.
    macros.insert_back_macro("\\vdots", text("\\mathord{\\varvdots\\rule{0pt}{15pt}}"));
    macros.insert_back_macro("\u{22ee}", text("\\vdots"));

    // amsmath.sty
    // http://mirrors.concertpass.com/tex-archive/macros/latex/required/amsmath/amsmath.pdf

    // Italic Greek capital letters.  AMS defines these with \DeclareMathSymbol,
    // but they are equivalent to \mathit{\Letter}.
    macros.insert_back_macro("\\varGamma", text("\\mathit{\\Gamma}"));
    macros.insert_back_macro("\\varDelta", text("\\mathit{\\Delta}"));
    macros.insert_back_macro("\\varTheta", text("\\mathit{\\Theta}"));
    macros.insert_back_macro("\\varLambda", text("\\mathit{\\Lambda}"));
    macros.insert_back_macro("\\varXi", text("\\mathit{\\Xi}"));
    macros.insert_back_macro("\\varPi", text("\\mathit{\\Pi}"));
    macros.insert_back_macro("\\varSigma", text("\\mathit{\\Sigma}"));
    macros.insert_back_macro("\\varUpsilon", text("\\mathit{\\Upsilon}"));
    macros.insert_back_macro("\\varPhi", text("\\mathit{\\Phi}"));
    macros.insert_back_macro("\\varPsi", text("\\mathit{\\Psi}"));
    macros.insert_back_macro("\\varOmega", text("\\mathit{\\Omega}"));

    //\newcommand{\substack}[1]{\subarray{c}#1\endsubarray}
    macros.insert_back_macro("\\substack", text("\\begin{subarray}{c}#1\\end{subarray}"));

    // \renewcommand{\colon}{\nobreak\mskip2mu\mathpunct{}\nonscript
    // \mkern-\thinmuskip{:}\mskip6muplus1mu\relax}
    macros.insert_back_macro(
        "\\colon",
        text("\\nobreak\\mskip2mu\\mathpunct{}\\mathchoice{\\mkern-3mu}{\\mkern-3mu}{}{}{:}\\mskip6mu\\relax"),
    );

    // \newcommand{\boxed}[1]{\fbox{\m@th$\displaystyle#1$}}
    macros.insert_back_macro("\\boxed", text("\\fbox{$\\displaystyle{#1}$}"));

    // \def\iff{\DOTSB\;\Longleftrightarrow\;}
    // \def\implies{\DOTSB\;\Longrightarrow\;}
    // \def\impliedby{\DOTSB\;\Longleftarrow\;}
    macros.insert_back_macro("\\iff", text("\\DOTSB\\;\\Longleftrightarrow\\;"));
    macros.insert_back_macro("\\implies", text("\\DOTSB\\;\\Longrightarrow\\;"));
    macros.insert_back_macro("\\impliedby", text("\\DOTSB\\;\\Longleftarrow\\;"));

    // AMSMath's automatic \dots, based on \mdots@@ macro.
    const DOTS_BY_TOKEN: &'static [(&'static str, &'static str)] = &[
        (",", "\\dotsc"),
        ("\\not", "\\dotsb"),
        // \keybin@ checks for the following:
        ("+", "\\dotsb"),
        ("=", "\\dotsb"),
        ("<", "\\dotsb"),
        (">", "\\dotsb"),
        ("-", "\\dotsb"),
        ("*", "\\dotsb"),
        (":", "\\dotsb"),
        // Symbols whose definition starts with \DOTSB:
        ("\\DOTSB", "\\dotsb"),
        ("\\coprod", "\\dotsb"),
        ("\\bigvee", "\\dotsb"),
        ("\\bigwedge", "\\dotsb"),
        ("\\biguplus", "\\dotsb"),
        ("\\bigcap", "\\dotsb"),
        ("\\bigcup", "\\dotsb"),
        ("\\prod", "\\dotsb"),
        ("\\sum", "\\dotsb"),
        ("\\bigotimes", "\\dotsb"),
        ("\\bigoplus", "\\dotsb"),
        ("\\bigodot", "\\dotsb"),
        ("\\bigsqcup", "\\dotsb"),
        ("\\And", "\\dotsb"),
        ("\\longrightarrow", "\\dotsb"),
        ("\\Longrightarrow", "\\dotsb"),
        ("\\longleftarrow", "\\dotsb"),
        ("\\Longleftarrow", "\\dotsb"),
        ("\\longleftrightarrow", "\\dotsb"),
        ("\\Longleftrightarrow", "\\dotsb"),
        ("\\mapsto", "\\dotsb"),
        ("\\longmapsto", "\\dotsb"),
        ("\\hookrightarrow", "\\dotsb"),
        ("\\doteq", "\\dotsb"),
        // Symbols whose definition starts with \mathbin:
        ("\\mathbin", "\\dotsb"),
        // Symbols whose definition starts with \mathrel:
        ("\\mathrel", "\\dotsb"),
        ("\\relbar", "\\dotsb"),
        ("\\Relbar", "\\dotsb"),
        ("\\xrightarrow", "\\dotsb"),
        ("\\xleftarrow", "\\dotsb"),
        // Symbols whose definition starts with \DOTSI:
        ("\\DOTSI", "\\dotsi"),
        ("\\int", "\\dotsi"),
        ("\\oint", "\\dotsi"),
        ("\\iint", "\\dotsi"),
        ("\\iiint", "\\dotsi"),
        ("\\iiiint", "\\dotsi"),
        ("\\idotsint", "\\dotsi"),
        // Symbols whose definition starts with \DOTSX:
        ("\\DOTSX", "\\dotsx"),
    ];

    macros.insert_back_macro(
        "\\dots",
        f(|exp| {
            // TODO: If used in text mode, should expand to \textellipsis.
            // However, in KaTeX, \textellipsis and \ldots behave the same
            // (in text mode), and it's unlikely we'd see any of the math commands
            // that affect the behavior of \dots when in text mode.  So fine for now
            // (until we support \ifmmode ... \else ... \fi).

            let next = exp.expand_after_future()?.content.as_ref();

            let the_dots = if let Some(dots) = find_assoc_data(DOTS_BY_TOKEN, next) {
                dots
            } else if next.starts_with("\\not") {
                "\\dotsb"
            } else if let Some(sym) = symbols::SYMBOLS.get(Mode::Math, &next) {
                if matches!(sym.group, Group::Atom(Atom::Bin) | Group::Atom(Atom::Rel)) {
                    "\\dotsb"
                } else {
                    "\\dotsc"
                }
            } else {
                "\\dotso"
            };

            Ok(MacroVal::Text(Cow::Borrowed(the_dots)))
        }),
    );

    const SPACE_AFTER_DOTS: &'static [&'static str] = &[
        // \rightdelim@ checks for the following:
        ")",
        "]",
        "\\rbrack",
        "\\}",
        "\\rbrace",
        "\\rangle",
        "\\rceil",
        "\\rfloor",
        "\\rgroup",
        "\\rmoustache",
        "\\right",
        "\\bigr",
        "\\biggr",
        "\\Bigr",
        "\\Biggr",
        // \extra@ also tests for the following:
        "$",
        // \extrap@ checks for the following:
        ";",
        ".",
        ",",
    ];

    macros.insert_back_macro(
        "\\dotso",
        f(|exp| {
            let next = exp.future()?.content.as_ref();
            if SPACE_AFTER_DOTS.contains(&next) {
                Ok(MacroVal::Text(Cow::Borrowed("\\ldots\\,")))
            } else {
                Ok(MacroVal::Text(Cow::Borrowed("\\ldots")))
            }
        }),
    );

    macros.insert_back_macro(
        "\\dotsc",
        f(|exp| {
            let next = exp.future()?.content.as_ref();
            // \dotsc uses \extra@ but not \extrap@, instead specially checking for
            // ';' and '.', but doesn't check for ','.
            if next != "," && SPACE_AFTER_DOTS.contains(&next) {
                Ok(MacroVal::Text(Cow::Borrowed("\\ldots\\,")))
            } else {
                Ok(MacroVal::Text(Cow::Borrowed("\\ldots")))
            }
        }),
    );

    macros.insert_back_macro(
        "\\cdots",
        f(|exp| {
            let next = exp.future()?.content.as_ref();
            if SPACE_AFTER_DOTS.contains(&next) {
                Ok(MacroVal::Text(Cow::Borrowed("\\@cdots\\,")))
            } else {
                Ok(MacroVal::Text(Cow::Borrowed("\\@cdots")))
            }
        }),
    );

    macros.insert_back_macro("\\dotsb", text("\\cdots"));
    macros.insert_back_macro("\\dotsm", text("\\cdots"));
    macros.insert_back_macro("\\dotsi", text("\\!\\cdots"));
    // amsmath doesn't actually define \dotsx, but \dots followed by a macro
    // starting with \DOTSX implies \dotso, and then \extra@ detects this case
    // and forces the added `\,`.
    macros.insert_back_macro("\\dotsx", text("\\ldots\\,"));

    // \let\DOTSI\relax
    // \let\DOTSB\relax
    // \let\DOTSX\relax
    macros.insert_back_macro("\\DOTSI", text("\\relax"));
    macros.insert_back_macro("\\DOTSB", text("\\relax"));
    macros.insert_back_macro("\\DOTSX", text("\\relax"));

    // Spacing, based on amsmath.sty's override of LaTeX defaults
    // \DeclareRobustCommand{\tmspace}[3]{%
    //   \ifmmode\mskip#1#2\else\kern#1#3\fi\relax}
    macros.insert_back_macro(
        "\\tmspace",
        text("\\TextOrMath{\\kern#1#3}{\\mskip#1#2}\\relax"),
    );
    // Math/text spacing (explicit TextOrMath to mirror KaTeX behavior)
    macros.insert_back_macro("\\,", text("\\TextOrMath{\\kern.1667em}{\\mskip3mu}"));
    // \let\thinspace\,
    macros.insert_back_macro("\\thinspace", text("\\,"));
    // \def\>{\mskip\medmuskip}
    macros.insert_back_macro("\\>", text("\\mskip{4mu}"));
    macros.insert_back_macro("\\:", text("\\TextOrMath{\\kern.2222em}{\\mskip4mu}"));
    // \let\medspace\:
    macros.insert_back_macro("\\medspace", text("\\:"));
    macros.insert_back_macro("\\;", text("\\TextOrMath{\\kern.2777em}{\\mskip5mu}"));
    // \let\thickspace\;
    macros.insert_back_macro("\\thickspace", text("\\;"));
    macros.insert_back_macro("\\!", text("\\TextOrMath{\\kern-.1667em}{\\mskip-3mu}"));
    // \let\negthinspace\!
    macros.insert_back_macro("\\negthinspace", text("\\!"));
    macros.insert_back_macro("\\negmedspace", text("\\TextOrMath{\\kern-.2222em}{\\mskip-4mu}"));
    macros.insert_back_macro("\\negthickspace", text("\\TextOrMath{\\kern-.2777em}{\\mskip-5mu}"));
    // \def\enspace{\kern.5em }
    macros.insert_back_macro("\\enspace", text("\\kern.5em "));
    // \def\enskip{\hskip.5em\relax}
    macros.insert_back_macro("\\enskip", text("\\hskip.5em\\relax"));
    // \def\quad{\hskip1em\relax}
    macros.insert_back_macro("\\quad", text("\\hskip1em\\relax"));
    // \def\qquad{\hskip2em\relax}
    macros.insert_back_macro("\\qquad", text("\\hskip2em\\relax"));

    // \tag@in@display form of \tag
    macros.insert_back_macro("\\tag", text("\\@ifstar\\tag@literal\\tag@paren"));
    macros.insert_back_macro("\\tag@paren", text("\\tag@literal{({#1})}"));
    macros.insert_back_macro(
        "\\tag@literal",
        f(|exp| {
            if exp.macros.get_back_macro("\\df@tag").is_some() {
                return Err(ParseError::MultipleTag);
            }
            Ok(MacroVal::Text(Cow::Borrowed("\\gdef\\df@tag{\\text{#1}}")))
        }),
    );

    // \renewcommand{\bmod}{\nonscript\mskip-\medmuskip\mkern5mu\mathbin
    //   {\operator@font mod}\penalty900
    //   \mkern5mu\nonscript\mskip-\medmuskip}
    // \newcommand{\pod}[1]{\allowbreak
    //   \if@display\mkern18mu\else\mkern8mu\fi(#1)}
    // \renewcommand{\pmod}[1]{\pod{{\operator@font mod}\mkern6mu#1}}
    // \newcommand{\mod}[1]{\allowbreak\if@display\mkern18mu
    //   \else\mkern12mu\fi{\operator@font mod}\,\,#1}
    // TODO: math mode should use \medmuskip = 4mu plus 2mu minus 4mu
    macros.insert_back_macro(
        "\\bmod",
        text("\\mathchoice{\\mskip1mu}{\\mskip1mu}{\\mskip5mu}{\\mskip5mu}\\mathbin{\\rm mod}\\mathchoice{\\mskip1mu}{\\mskip1mu}{\\mskip5mu}{\\mskip5mu}"),
    );
    macros.insert_back_macro(
        "\\pod",
        text("\\allowbreak\\mathchoice{\\mkern18mu}{\\mkern8mu}{\\mkern8mu}{\\mkern8mu}(#1)"),
    );
    macros.insert_back_macro("\\pmod", text("\\pod{{\\rm mod}\\mkern6mu#1}"));
    macros.insert_back_macro(
        "\\mod",
        text("\\allowbreak\\mathchoice{\\mkern18mu}{\\mkern12mu}{\\mkern12mu}{\\mkern12mu}{\\rm mod}\\,\\,#1"),
    );

    // \pmb    --   A simulation of bold.
    // The version in ambsy.sty works by typesetting three copies of the argument
    // with small offsets. We use two copies. We omit the vertical offset because
    // of rendering problems that makeVList encounters in Safari.
    macros.insert_back_macro(
        "\\pmb",
        text("\\html@mathml{\\@binrel{#1}{\\mathrlap{#1}\\kern0.5px#1}}{\\mathbf{#1}}"),
    );

    // LaTeX source2e

    // \expandafter\let\expandafter\@normalcr
    //     \csname\expandafter\@gobble\string\\ \endcsname
    // \DeclareRobustCommand\newline{\@normalcr\relax}
    macros.insert_back_macro("\\newline", text("\\\\\\relax"));

    // \def\TeX{T\kern-.1667em\lower.5ex\hbox{E}\kern-.125emX\@}
    // TODO: Doesn't normally work in math mode because \@ fails.  KaTeX doesn't
    // support \@ yet, so that's omitted, and we add \text so that the result
    // doesn't look funny in math mode.
    macros.insert_back_macro(
        "\\TeX",
        text("\\textrm{\\html@mathml{T\\kern-.1667em\\raisebox{-.5ex}{E}\\kern-.125emX}{TeX}}"),
    );

    // \DeclareRobustCommand{\LaTeX}{L\kern-.36em%
    //         {\sbox\z@ T%
    //          \vbox to\ht\z@{\hbox{\check@mathfonts
    //                               \fontsize\sf@size\z@
    //                               \math@fontsfalse\selectfont
    //                               A}%
    //                         \vss}%
    //         }%
    //         \kern-.15em%
    //         \TeX}
    // This code aligns the top of the A with the T (from the perspective of TeX's
    // boxes, though visually the A appears to extend above slightly).
    // We compute the corresponding \raisebox when A is rendered in \normalsize
    // \scriptstyle, which has a scale factor of 0.7 (see Options.js).
    let metric = font_metrics_data::get_metric("Main-Regular").unwrap();
    let t_data = find_assoc_data(metric, char_code_for('T')).unwrap();
    let a_data = find_assoc_data(metric, char_code_for('A')).unwrap();
    let latex_raise_a = make_em(t_data[1] - 0.7 * a_data[1]);

    let latex = format!("\\textrm{{\\html@mathml{{L\\kern-.36em\\raisebox{{{latex_raise_a}}}{{\\scriptstyle A}}\\kern-.15em\\TeX}}{{LaTeX}}}}");
    macros.insert_back_macro("\\LaTeX", Arc::new(MacroReplace::Text(latex)));

    // New KaTeX logo based on tweaking LaTeX logo
    let katex = format!("\\textrm{{\\html@mathml{{K\\kern-.17em\\raisebox{{{latex_raise_a}}}{{\\scriptstyle A}}\\kern-.15em\\TeX}}{{KaTeX}}}}");
    macros.insert_back_macro("\\KaTeX", Arc::new(MacroReplace::Text(katex)));

    // \DeclareRobustCommand\hspace{\@ifstar\@hspacer\@hspace}
    // \def\@hspace#1{\hskip  #1\relax}
    // \def\@hspacer#1{\vrule \@width\z@\nobreak
    //                 \hskip #1\hskip \z@skip}
    macros.insert_back_macro("\\hspace", text("\\@ifstar\\@hspacer\\@hspace"));
    macros.insert_back_macro("\\@hspace", text("\\hskip #1\\relax"));
    macros.insert_back_macro("\\@hspacer", text("\\rule{0pt}{0pt}\\hskip #1\\relax"));

    // mathtools.sty

    //\providecommand\ordinarycolon{:}
    macros.insert_back_macro("\\ordinarycolon", text(":"));
    //\def\vcentcolon{\mathrel{\mathop\ordinarycolon}}
    //TODO(edemaine): Not yet centered. Fix via \raisebox or #726
    macros.insert_back_macro("\\vcentcolon", text("\\mathrel{\\mathop\\ordinarycolon}"));
    // \providecommand*\dblcolon{\vcentcolon\mathrel{\mkern-.9mu}\vcentcolon}
    macros.insert_back_macro("\\dblcolon", text("\\html@mathml{\\mathrel{\\vcentcolon\\mathrel{\\mkern-.9mu}\\vcentcolon}}{\\mathop{\\char\"2237}}"));
    // \providecommand*\coloneqq{\vcentcolon\mathrel{\mkern-1.2mu}=}
    macros.insert_back_macro("\\coloneqq", text("\\html@mathml{\\mathrel{\\vcentcolon\\mathrel{\\mkern-1.2mu}=}}{\\mathop{\\char\"2254}}")); // ≔
                                                                                                                                             // \providecommand*\Coloneqq{\dblcolon\mathrel{\mkern-1.2mu}=}
    macros.insert_back_macro("\\Coloneqq", text("\\html@mathml{\\mathrel{\\dblcolon\\mathrel{\\mkern-1.2mu}=}}{\\mathop{\\char\"2237\\char\"3d}}"));
    // \providecommand*\coloneq{\vcentcolon\mathrel{\mkern-1.2mu}\mathrel{-}}
    macros.insert_back_macro("\\coloneq", text("\\html@mathml{\\mathrel{\\vcentcolon\\mathrel{\\mkern-1.2mu}\\mathrel{-}}}{\\mathop{\\char\"3a\\char\"2212}}"));
    // \providecommand*\Coloneq{\dblcolon\mathrel{\mkern-1.2mu}\mathrel{-}}
    macros.insert_back_macro("\\Coloneq", text("\\html@mathml{\\mathrel{\\dblcolon\\mathrel{\\mkern-1.2mu}\\mathrel{-}}}{\\mathop{\\char\"2237\\char\"2212}}"));
    // \providecommand*\eqqcolon{=\mathrel{\mkern-1.2mu}\vcentcolon}
    macros.insert_back_macro("\\eqqcolon", text("\\html@mathml{\\mathrel{=\\mathrel{\\mkern-1.2mu}\\vcentcolon}}{\\mathop{\\char\"2255}}")); // ≕
                                                                                                                                             // \providecommand*\Eqqcolon{=\mathrel{\mkern-1.2mu}\dblcolon}
    macros.insert_back_macro("\\Eqqcolon", text("\\html@mathml{\\mathrel{=\\mathrel{\\mkern-1.2mu}\\dblcolon}}{\\mathop{\\char\"3d\\char\"2237}}"));
    // \providecommand*\eqcolon{\mathrel{-}\mathrel{\mkern-1.2mu}\vcentcolon}
    macros.insert_back_macro("\\eqcolon", text("\\html@mathml{\\mathrel{\\mathrel{-}\\mathrel{\\mkern-1.2mu}\\vcentcolon}}{\\mathop{\\char\"2239}}"));
    // \providecommand*\Eqcolon{\mathrel{-}\mathrel{\mkern-1.2mu}\dblcolon}
    macros.insert_back_macro("\\Eqcolon", text("\\html@mathml{\\mathrel{\\mathrel{-}\\mathrel{\\mkern-1.2mu}\\dblcolon}}{\\mathop{\\char\"2212\\char\"2237}}"));
    // \providecommand*\colonapprox{\vcentcolon\mathrel{\mkern-1.2mu}\approx}
    macros.insert_back_macro("\\colonapprox", text("\\html@mathml{\\mathrel{\\vcentcolon\\mathrel{\\mkern-1.2mu}\\approx}}{\\mathop{\\char\"3a\\char\"2248}}"));
    // \providecommand*\Colonapprox{\dblcolon\mathrel{\mkern-1.2mu}\approx}
    macros.insert_back_macro("\\Colonapprox", text("\\html@mathml{\\mathrel{\\dblcolon\\mathrel{\\mkern-1.2mu}\\approx}}{\\mathop{\\char\"2237\\char\"2248}}"));
    // \providecommand*\colonsim{\vcentcolon\mathrel{\mkern-1.2mu}\sim}
    macros.insert_back_macro("\\colonsim", text("\\html@mathml{\\mathrel{\\vcentcolon\\mathrel{\\mkern-1.2mu}\\sim}}{\\mathop{\\char\"3a\\char\"223c}}"));
    // \providecommand*\Colonsim{\dblcolon\mathrel{\mkern-1.2mu}\sim}
    macros.insert_back_macro("\\Colonsim", text("\\html@mathml{\\mathrel{\\dblcolon\\mathrel{\\mkern-1.2mu}\\sim}}{\\mathop{\\char\"2237\\char\"223c}}"));

    // Some Unicode characters are implemented with macros to mathtools functions.
    macros.insert_back_macro("\u{2237}", text("\\dblcolon")); // ::
    macros.insert_back_macro("\u{2239}", text("\\eqcolon")); // -:
    macros.insert_back_macro("\u{2254}", text("\\coloneqq")); // :=
    macros.insert_back_macro("\u{2255}", text("\\eqqcolon")); // =:
    macros.insert_back_macro("\u{2A74}", text("\\Coloneqq")); // ::=

    // colonequals.sty

    // Alternate names for mathtools's macros:
    macros.insert_back_macro("\\ratio", text("\\vcentcolon"));
    macros.insert_back_macro("\\coloncolon", text("\\dblcolon"));
    macros.insert_back_macro("\\colonequals", text("\\coloneqq"));
    macros.insert_back_macro("\\coloncolonequals", text("\\Coloneqq"));
    macros.insert_back_macro("\\equalscolon", text("\\eqqcolon"));
    macros.insert_back_macro("\\equalscoloncolon", text("\\Eqqcolon"));
    macros.insert_back_macro("\\colonminus", text("\\coloneq"));
    macros.insert_back_macro("\\coloncolonminus", text("\\Coloneq"));
    macros.insert_back_macro("\\minuscolon", text("\\eqcolon"));
    macros.insert_back_macro("\\minuscoloncolon", text("\\Eqcolon"));
    // \colonapprox name is same in mathtools and colonequals.
    macros.insert_back_macro("\\coloncolonapprox", text("\\Colonapprox"));
    // \colonsim name is same in mathtools and colonequals.
    macros.insert_back_macro("\\coloncolonsim", text("\\Colonsim"));

    // Additional macros, implemented by analogy with mathtools definitions:
    macros.insert_back_macro(
        "\\simcolon",
        text("\\mathrel{\\sim\\mathrel{\\mkern-1.2mu}\\vcentcolon}"),
    );
    macros.insert_back_macro(
        "\\simcoloncolon",
        text("\\mathrel{\\sim\\mathrel{\\mkern-1.2mu}\\dblcolon}"),
    );
    macros.insert_back_macro(
        "\\approxcolon",
        text("\\mathrel{\\approx\\mathrel{\\mkern-1.2mu}\\vcentcolon}"),
    );
    macros.insert_back_macro(
        "\\approxcoloncolon",
        text("\\mathrel{\\approx\\mathrel{\\mkern-1.2mu}\\dblcolon}"),
    );

    // Present in newtxmath, pxfonts and txfonts
    macros.insert_back_macro(
        "\\notni",
        text("\\html@mathml{\\not\\ni}{\\mathrel{\\char`\u{220C}}}"),
    );
    macros.insert_back_macro("\\limsup", text("\\DOTSB\\operatorname*{lim\\,sup}"));
    macros.insert_back_macro("\\liminf", text("\\DOTSB\\operatorname*{lim\\,inf}"));

    //////////////////////////////////////////////////////////////////////
    // From amsopn.sty
    macros.insert_back_macro("\\injlim", text("\\DOTSB\\operatorname*{inj\\,lim}"));
    macros.insert_back_macro("\\projlim", text("\\DOTSB\\operatorname*{proj\\,lim}"));
    macros.insert_back_macro(
        "\\varlimsup",
        text("\\DOTSB\\operatorname*{\\overline{lim}}"),
    );
    macros.insert_back_macro(
        "\\varliminf",
        text("\\DOTSB\\operatorname*{\\underline{lim}}"),
    );
    macros.insert_back_macro(
        "\\varinjlim",
        text("\\DOTSB\\operatorname*{\\underrightarrow{lim}}"),
    );
    macros.insert_back_macro(
        "\\varprojlim",
        text("\\DOTSB\\operatorname*{\\underleftarrow{lim}}"),
    );

    //////////////////////////////////////////////////////////////////////
    // MathML alternates for KaTeX glyphs in the Unicode private area
    macros.insert_back_macro("\\gvertneqq", text("\\html@mathml{\\@gvertneqq}{\u{2269}}"));
    macros.insert_back_macro("\\lvertneqq", text("\\html@mathml{\\@lvertneqq}{\u{2268}}"));
    macros.insert_back_macro("\\ngeqq", text("\\html@mathml{\\@ngeqq}{\u{2271}}"));
    macros.insert_back_macro("\\ngeqslant", text("\\html@mathml{\\@ngeqslant}{\u{2271}}"));
    macros.insert_back_macro("\\nleqq", text("\\html@mathml{\\@nleqq}{\u{2270}}"));
    macros.insert_back_macro("\\nleqslant", text("\\html@mathml{\\@nleqslant}{\u{2270}}"));
    macros.insert_back_macro("\\nshortmid", text("\\html@mathml{\\@nshortmid}{∤}"));
    macros.insert_back_macro(
        "\\nshortparallel",
        text("\\html@mathml{\\@nshortparallel}{∦}"),
    );
    macros.insert_back_macro(
        "\\nsubseteqq",
        text("\\html@mathml{\\@nsubseteqq}{\u{2288}}"),
    );
    macros.insert_back_macro(
        "\\nsupseteqq",
        text("\\html@mathml{\\@nsupseteqq}{\u{2289}}"),
    );
    macros.insert_back_macro("\\varsubsetneq", text("\\html@mathml{\\@varsubsetneq}{⊊}"));
    macros.insert_back_macro(
        "\\varsubsetneqq",
        text("\\html@mathml{\\@varsubsetneqq}{⫋}"),
    );
    macros.insert_back_macro("\\varsupsetneq", text("\\html@mathml{\\@varsupsetneq}{⊋}"));
    macros.insert_back_macro(
        "\\varsupsetneqq",
        text("\\html@mathml{\\@varsupsetneqq}{⫌}"),
    );
    macros.insert_back_macro("\\imath", text("\\html@mathml{\\@imath}{\u{0131}}"));
    macros.insert_back_macro("\\jmath", text("\\html@mathml{\\@jmath}{\u{0237}}"));

    //////////////////////////////////////////////////////////////////////
    // stmaryrd and semantic

    // The stmaryrd and semantic packages render the next four items by calling a
    // glyph. Those glyphs do not exist in the KaTeX fonts. Hence the macros.

    macros.insert_back_macro(
        "\\llbracket",
        text("\\html@mathml{\\mathopen{[\\mkern-3.2mu[}}{\\mathopen{\\char`\u{27e6}}}"),
    );
    macros.insert_back_macro(
        "\\rrbracket",
        text("\\html@mathml{\\mathclose{]\\mkern-3.2mu]}}{\\mathclose{\\char`\u{27e7}}}"),
    );

    macros.insert_back_macro("\u{27e6}", text("\\llbracket")); // blackboard bold [
    macros.insert_back_macro("\u{27e7}", text("\\rrbracket")); // blackboard bold ]

    macros.insert_back_macro(
        "\\lBrace",
        text("\\html@mathml{\\mathopen{\\{\\mkern-3.2mu[}}{\\mathopen{\\char`\u{2983}}}"),
    );
    macros.insert_back_macro(
        "\\rBrace",
        text("\\html@mathml{\\mathclose{]\\mkern-3.2mu\\}}}{\\mathclose{\\char`\u{2984}}}"),
    );

    macros.insert_back_macro("\u{2983}", text("\\lBrace")); // blackboard bold {
    macros.insert_back_macro("\u{2984}", text("\\rBrace")); // blackboard bold }

    // TODO: Create variable sized versions of the last two items. I believe that
    // will require new font glyphs.

    // The stmaryrd function `\minuso` provides a "Plimsoll" symbol that
    // superimposes the characters \circ and \mathminus. Used in chemistry.
    macros.insert_back_macro(
        "\\minuso",
        text("\\mathbin{\\html@mathml{{\\mathrlap{\\mathchoice{\\kern{0.145em}}{\\kern{0.145em}}{\\kern{0.1015em}}{\\kern{0.0725em}}\\circ}{-}}}{\\char`⦵}}"),
    );
    macros.insert_back_macro("⦵", text("\\minuso"));

    //////////////////////////////////////////////////////////////////////
    // texvc.sty

    // The texvc package contains macros available in mediawiki pages.
    // We omit the functions deprecated at
    // https://en.wikipedia.org/wiki/Help:Displaying_a_formula#Deprecated_syntax

    // We also omit texvc's \O, which conflicts with \text{\O}

    macros.insert_back_macro("\\darr", text("\\downarrow"));
    macros.insert_back_macro("\\dArr", text("\\Downarrow"));
    macros.insert_back_macro("\\Darr", text("\\Downarrow"));
    macros.insert_back_macro("\\lang", text("\\langle"));
    macros.insert_back_macro("\\rang", text("\\rangle"));
    macros.insert_back_macro("\\uarr", text("\\uparrow"));
    macros.insert_back_macro("\\uArr", text("\\Uparrow"));
    macros.insert_back_macro("\\Uarr", text("\\Uparrow"));
    macros.insert_back_macro("\\N", text("\\mathbb{N}"));
    macros.insert_back_macro("\\R", text("\\mathbb{R}"));
    macros.insert_back_macro("\\Z", text("\\mathbb{Z}"));
    macros.insert_back_macro("\\alef", text("\\aleph"));
    macros.insert_back_macro("\\alefsym", text("\\aleph"));
    macros.insert_back_macro("\\Alpha", text("\\mathrm{A}"));
    macros.insert_back_macro("\\Beta", text("\\mathrm{B}"));
    macros.insert_back_macro("\\bull", text("\\bullet"));
    macros.insert_back_macro("\\Chi", text("\\mathrm{X}"));
    macros.insert_back_macro("\\clubs", text("\\clubsuit"));
    macros.insert_back_macro("\\cnums", text("\\mathbb{C}"));
    macros.insert_back_macro("\\Complex", text("\\mathbb{C}"));
    macros.insert_back_macro("\\Dagger", text("\\ddagger"));
    macros.insert_back_macro("\\diamonds", text("\\diamondsuit"));
    macros.insert_back_macro("\\empty", text("\\emptyset"));
    macros.insert_back_macro("\\Epsilon", text("\\mathrm{E}"));
    macros.insert_back_macro("\\Eta", text("\\mathrm{H}"));
    macros.insert_back_macro("\\exist", text("\\exists"));
    macros.insert_back_macro("\\harr", text("\\leftrightarrow"));
    macros.insert_back_macro("\\hArr", text("\\Leftrightarrow"));
    macros.insert_back_macro("\\Harr", text("\\Leftrightarrow"));
    macros.insert_back_macro("\\hearts", text("\\heartsuit"));
    macros.insert_back_macro("\\image", text("\\Im"));
    macros.insert_back_macro("\\infin", text("\\infty"));
    macros.insert_back_macro("\\Iota", text("\\mathrm{I}"));
    macros.insert_back_macro("\\isin", text("\\in"));
    macros.insert_back_macro("\\Kappa", text("\\mathrm{K}"));
    macros.insert_back_macro("\\larr", text("\\leftarrow"));
    macros.insert_back_macro("\\lArr", text("\\Leftarrow"));
    macros.insert_back_macro("\\Larr", text("\\Leftarrow"));
    macros.insert_back_macro("\\lrarr", text("\\leftrightarrow"));
    macros.insert_back_macro("\\lrArr", text("\\Leftrightarrow"));
    macros.insert_back_macro("\\Lrarr", text("\\Leftrightarrow"));
    macros.insert_back_macro("\\Mu", text("\\mathrm{M}"));
    macros.insert_back_macro("\\natnums", text("\\mathbb{N}"));
    macros.insert_back_macro("\\Nu", text("\\mathrm{N}"));
    macros.insert_back_macro("\\Omicron", text("\\mathrm{O}"));
    macros.insert_back_macro("\\plusmn", text("\\pm"));
    macros.insert_back_macro("\\rarr", text("\\rightarrow"));
    macros.insert_back_macro("\\rArr", text("\\Rightarrow"));
    macros.insert_back_macro("\\Rarr", text("\\Rightarrow"));
    macros.insert_back_macro("\\real", text("\\Re"));
    macros.insert_back_macro("\\reals", text("\\mathbb{R}"));
    macros.insert_back_macro("\\Reals", text("\\mathbb{R}"));
    macros.insert_back_macro("\\Rho", text("\\mathrm{P}"));
    macros.insert_back_macro("\\sdot", text("\\cdot"));
    macros.insert_back_macro("\\sect", text("\\S"));
    macros.insert_back_macro("\\spades", text("\\spadesuit"));
    macros.insert_back_macro("\\sub", text("\\subset"));
    macros.insert_back_macro("\\sube", text("\\subseteq"));
    macros.insert_back_macro("\\supe", text("\\supseteq"));
    macros.insert_back_macro("\\Tau", text("\\mathrm{T}"));
    macros.insert_back_macro("\\thetasym", text("\\vartheta"));
    // TODO: defineMacro("\\varcoppa", "\\\mbox{\\coppa}");
    macros.insert_back_macro("\\weierp", text("\\wp"));
    macros.insert_back_macro("\\Zeta", text("\\mathrm{Z}"));

    //////////////////////////////////////////////////////////////////////
    // statmath.sty
    // https://ctan.math.illinois.edu/macros/latex/contrib/statmath/statmath.pdf

    macros.insert_back_macro("\\argmin", text("\\DOTSB\\operatorname*{arg\\,min}"));
    macros.insert_back_macro("\\argmax", text("\\DOTSB\\operatorname*{arg\\,max}"));
    macros.insert_back_macro(
        "\\plim",
        text("\\DOTSB\\mathop{\\operatorname{plim}}\\limits"),
    );

    //////////////////////////////////////////////////////////////////////
    // braket.sty
    // http://ctan.math.washington.edu/tex-archive/macros/latex/contrib/braket/braket.pdf

    macros.insert_back_macro("\\bra", text("\\mathinner{\\langle{#1}|}"));
    macros.insert_back_macro("\\ket", text("\\mathinner{|{#1}\\rangle}"));
    macros.insert_back_macro("\\braket", text("\\mathinner{\\langle{#1}\\rangle}"));
    macros.insert_back_macro("\\Bra", text("\\left\\langle#1\\right|"));
    macros.insert_back_macro("\\Ket", text("\\left|#1\\right\\rangle"));

    // TODO: is there a cleaner way to implement this?
    fn mid_macro<const ONE: bool, const DOUBLE: bool>(
        middle: Vec<Token<'static>>,
        middle_double: Vec<Token<'static>>,
        old_middle: Option<Arc<MacroReplace>>,
        old_middle_double: Option<Arc<MacroReplace>>,
    ) -> Arc<MacroReplace> {
        f(move |exp: &mut MacroExpander<'_, '_>| {
            if ONE {
                // only modify the first instance of | or \|
                exp.macros.set_letter_macro('|', old_middle.clone());
                if !middle_double.is_empty() {
                    exp.macros.set_back_macro("\\|", old_middle_double.clone());
                }
            }

            let mut doubled = DOUBLE;
            if !DOUBLE && !middle_double.is_empty() {
                // Mimic \@ifnextchar
                let next_token = exp.future()?;
                if next_token.content == "|" {
                    exp.pop_token()?;
                    doubled = true;
                }
            }

            let tokens = if doubled {
                middle_double.clone()
            } else {
                middle.clone()
            };

            Ok(MacroVal::Expansion(MacroExpansion::new(tokens, 0)))
        })
    }

    fn braket_helper<'a, 'f, const ONE: bool>(
        exp: &mut MacroExpander<'a, 'f>,
    ) -> Result<MacroVal<'a, 'static>, ParseError> {
        let left = exp.consume_arg()?.tokens;
        let middle = exp.consume_arg()?.tokens;
        let middle_double = exp.consume_arg()?.tokens;
        let right = exp.consume_arg()?.tokens;

        let old_middle = exp.macros.get_letter_macro('|').cloned();
        let old_middle_double = exp.macros.get_back_macro("\\|").cloned();

        exp.macros.begin_group();

        let middle: Vec<Token<'static>> = middle.into_iter().map(Token::into_owned).collect();
        let middle_double: Vec<Token<'static>> =
            middle_double.into_iter().map(Token::into_owned).collect();

        exp.macros.set_letter_macro(
            '|',
            Some(mid_macro::<ONE, false>(
                middle.clone(),
                middle_double.clone(),
                old_middle.clone(),
                old_middle_double.clone(),
            )),
        );
        if !middle_double.is_empty() {
            exp.macros.set_back_macro(
                "\\|",
                Some(mid_macro::<ONE, true>(
                    middle,
                    middle_double,
                    old_middle,
                    old_middle_double,
                )),
            );
        }

        let arg = exp.consume_arg()?.tokens;
        let mut expanded = exp.expand_tokens(right.into_iter().chain(arg).chain(left))?;
        exp.macros.end_group();

        expanded.reverse();

        Ok(MacroExpansion::new(expanded, 0).into())
    }

    macros.insert_back_macro("\\bra@ket", f(braket_helper::<false>));
    macros.insert_back_macro("\\bra@set", f(braket_helper::<true>));

    macros.insert_back_macro(
        "\\Braket",
        text("\\bra@ket{\\left\\langle}{\\,\\middle\\vert\\,}{\\,\\middle\\vert\\,}{\\right\\rangle}"),
    );
    macros.insert_back_macro(
        "\\Set",
        text("\\bra@set{\\left\\{\\:}{\\;\\middle\\vert\\;}{\\;\\middle\\Vert\\;}{\\:\\right\\}}"),
    );
    macros.insert_back_macro("\\set", text("\\bra@set{\\{\\,}{\\mid}{}{\\,\\}}"));
    // has no support for special || or \|

    //////////////////////////////////////////////////////////////////////
    // actuarialangle.dtx
    macros.insert_back_macro("\\angln", text("{\\angl n}"));

    // Custom Khan Academy colors, should be moved to an optional package
    macros.insert_back_macro("\\blue", text("\\textcolor{##6495ed}{#1}"));
    macros.insert_back_macro("\\orange", text("\\textcolor{##ffa500}{#1}"));
    macros.insert_back_macro("\\pink", text("\\textcolor{##ff00af}{#1}"));
    macros.insert_back_macro("\\red", text("\\textcolor{##df0030}{#1}"));
    macros.insert_back_macro("\\green", text("\\textcolor{##28ae7b}{#1}"));
    macros.insert_back_macro("\\gray", text("\\textcolor{gray}{#1}"));
    macros.insert_back_macro("\\purple", text("\\textcolor{##9d38bd}{#1}"));
    macros.insert_back_macro("\\blueA", text("\\textcolor{##ccfaff}{#1}"));
    macros.insert_back_macro("\\blueB", text("\\textcolor{##80f6ff}{#1}"));
    macros.insert_back_macro("\\blueC", text("\\textcolor{##63d9ea}{#1}"));
    macros.insert_back_macro("\\blueD", text("\\textcolor{##11accd}{#1}"));
    macros.insert_back_macro("\\blueE", text("\\textcolor{##0c7f99}{#1}"));
    macros.insert_back_macro("\\tealA", text("\\textcolor{##94fff5}{#1}"));
    macros.insert_back_macro("\\tealB", text("\\textcolor{##26edd5}{#1}"));
    macros.insert_back_macro("\\tealC", text("\\textcolor{##01d1c1}{#1}"));
    macros.insert_back_macro("\\tealD", text("\\textcolor{##01a995}{#1}"));
    macros.insert_back_macro("\\tealE", text("\\textcolor{##208170}{#1}"));
    macros.insert_back_macro("\\greenA", text("\\textcolor{##b6ffb0}{#1}"));
    macros.insert_back_macro("\\greenB", text("\\textcolor{##8af281}{#1}"));
    macros.insert_back_macro("\\greenC", text("\\textcolor{##74cf70}{#1}"));
    macros.insert_back_macro("\\greenD", text("\\textcolor{##1fab54}{#1}"));
    macros.insert_back_macro("\\greenE", text("\\textcolor{##0d923f}{#1}"));
    macros.insert_back_macro("\\goldA", text("\\textcolor{##ffd0a9}{#1}"));
    macros.insert_back_macro("\\goldB", text("\\textcolor{##ffbb71}{#1}"));
    macros.insert_back_macro("\\goldC", text("\\textcolor{##ff9c39}{#1}"));
    macros.insert_back_macro("\\goldD", text("\\textcolor{##e07d10}{#1}"));
    macros.insert_back_macro("\\goldE", text("\\textcolor{##a75a05}{#1}"));
    macros.insert_back_macro("\\redA", text("\\textcolor{##fca9a9}{#1}"));
    macros.insert_back_macro("\\redB", text("\\textcolor{##ff8482}{#1}"));
    macros.insert_back_macro("\\redC", text("\\textcolor{##f9685d}{#1}"));
    macros.insert_back_macro("\\redD", text("\\textcolor{##e84d39}{#1}"));
    macros.insert_back_macro("\\redE", text("\\textcolor{##bc2612}{#1}"));
    macros.insert_back_macro("\\maroonA", text("\\textcolor{##ffbde0}{#1}"));
    macros.insert_back_macro("\\maroonB", text("\\textcolor{##ff92c6}{#1}"));
    macros.insert_back_macro("\\maroonC", text("\\textcolor{##ed5fa6}{#1}"));
    macros.insert_back_macro("\\maroonD", text("\\textcolor{##ca337c}{#1}"));
    macros.insert_back_macro("\\maroonE", text("\\textcolor{##9e034e}{#1}"));
    macros.insert_back_macro("\\purpleA", text("\\textcolor{##ddd7ff}{#1}"));
    macros.insert_back_macro("\\purpleB", text("\\textcolor{##c6b9fc}{#1}"));
    macros.insert_back_macro("\\purpleC", text("\\textcolor{##aa87ff}{#1}"));
    macros.insert_back_macro("\\purpleD", text("\\textcolor{##7854ab}{#1}"));
    macros.insert_back_macro("\\purpleE", text("\\textcolor{##543b78}{#1}"));
    macros.insert_back_macro("\\mintA", text("\\textcolor{##f5f9e8}{#1}"));
    macros.insert_back_macro("\\mintB", text("\\textcolor{##edf2df}{#1}"));
    macros.insert_back_macro("\\mintC", text("\\textcolor{##e0e5cc}{#1}"));
    macros.insert_back_macro("\\grayA", text("\\textcolor{##f6f7f7}{#1}"));
    macros.insert_back_macro("\\grayB", text("\\textcolor{##f0f1f2}{#1}"));
    macros.insert_back_macro("\\grayC", text("\\textcolor{##e3e5e6}{#1}"));
    macros.insert_back_macro("\\grayD", text("\\textcolor{##d6d8da}{#1}"));
    macros.insert_back_macro("\\grayE", text("\\textcolor{##babec2}{#1}"));
    macros.insert_back_macro("\\grayF", text("\\textcolor{##888d93}{#1}"));
    macros.insert_back_macro("\\grayG", text("\\textcolor{##626569}{#1}"));
    macros.insert_back_macro("\\grayH", text("\\textcolor{##3b3e40}{#1}"));
    macros.insert_back_macro("\\grayI", text("\\textcolor{##21242c}{#1}"));
    macros.insert_back_macro("\\kaBlue", text("\\textcolor{##314453}{#1}"));
    macros.insert_back_macro("\\kaGreen", text("\\textcolor{##71B307}{#1}"));

    // Stub implementations for unported functions (parse-only placeholders)
    macros.insert_back_macro("\\left", text(""));
    macros.insert_back_macro("\\right", text(""));
    macros.insert_back_macro("\\overbrace", text("{#1}"));
    macros.insert_back_macro("\\underbrace", text("{#1}"));
    macros.insert_back_macro("\\phantom", text("{#1}"));
    macros.insert_back_macro("\\hphantom", text("{#1}"));
    macros.insert_back_macro("\\vphantom", text("{#1}"));
    macros.insert_back_macro("\\rule", text("{#1}{#2}"));

    macros
});

fn ch_to_digit(ch: char) -> Option<u16> {
    Some(match ch {
        '0'..='9' => ch as u16 - '0' as u16,
        'a'..='f' => ch as u16 - 'a' as u16 + 10,
        'A'..='F' => ch as u16 - 'A' as u16 + 10,
        _ => return None,
    })
}
