use std::{borrow::Cow, sync::Arc};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    expander::{is_implicit_command, BreakToken, MacroExpander, Mode},
    functions::{FunctionContext, FunctionSpec, Functions},
    lexer::{CategoryCode, Token},
    macr::{MacroReplace, Macros},
    parse_node::{
        AccentNode, AtomNode, Color, ColorNode, ColorTokenNode, NodeInfo, OrdGroupNode, ParseNode,
        ParseNodeType, RawNode, SizeNode, StylingNode, SupSubNode, TextNode, TextOrdNode,
        UnsupportedCmdParseNode, UrlNode, VerbNode,
    },
    symbols::{self, Group},
    unicode,
    unit::{self, Measurement},
    util::{
        first_ch_str, parse_rgb, parse_rgb_3, parse_rgba, ArgType, SourceLocation, Style, RGBA,
    },
};

// (?i) must be at the start for Rust's regex engine
static COLOR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new("(?i)^(?:#?(?:[a-f0-9]{3}|[a-f0-9]{6})|[a-z]+)$").unwrap()
});

// static SIX_HEX_COLOR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("^(?i)[0-9a-f]{6}").unwrap());

static SIZE_GROUP_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("^[-+]? *(?:$|\\d+|\\d+\\.\\d*|\\.\\d*) *[a-z]{0,2} *$").unwrap());
static SIZE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("([-+]?) *(\\d+(?:\\.\\d*)?|\\.\\d+) *([a-z]{2})").unwrap());

// Global replacement is handled by `replace_all`, so no `g` flag is needed.
static URL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\([#$%&~_^{}])").unwrap());

static SYMBOL_VERB_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("^\\\\verb[^a-zA-Z]").unwrap());

static COMBINING_DIACRITICAL_MARKS_END_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("[\u{0300}-\u{036f}]+$").unwrap());

// TODO: provide more debug information on these
#[derive(Debug, Clone)]
pub enum ParseError {
    Expected,
    ExpectedEndOfFile,
    UnexpectedEOF,
    /// (Pos, char)
    UnexpectedChar(usize, char),
    /// (name)
    UndefinedControlSequence(String),

    PrimitiveCantBeOptional,

    /// A mismatch between the number of delimiters and the numbers of args
    MismatchDelimitersArgsLength,
    /// The usage of the macro doesn't match its definition
    MismatchMacroDefinition,

    InvalidRegexMode,

    UnknownAccent,

    ExpectedLimitControls,
    DoubleSuperscript,
    DoubleSubscript,

    TooManyExpansions,

    IncompletePlaceholder,
    InvalidArgumentNumber,

    ExpectedGroup,

    OnlyOneInfixOperatorPerGroup,

    FunctionNoArguments,
    FunctionUnusableTextMode,
    FunctionUnusableMathMode,
    NoFunctionHandler,

    NullArgument,

    InvalidColor,
    InvalidSize,
    InvalidUnit,

    TagOnlyDisplayEquation,

    // Macro errors
    CharMissingArgument,
    CharInvalidBaseDigit,

    NewCommandFirstArgMustBeName,
    /// `\\newcommand`: Attempting to redefine command, use `\\renewcommand` instead
    NewCommandAttemptingToRedefine(String),
    /// `\\renewcommand`: Command does not exist, use `\\newcommand` instead
    NewCommandAttemptingToDefine(String),
    /// Multiple `\\tag` defs
    MultipleTag,
}

/// Configuration options for parsing.
/// Inherits several of the options that KaTeX would generate.
/// Does not have `output` since that is handled by other parts.
///
#[derive(Debug, Clone)]
pub struct ParserConfig {
    pub display_mode: bool,
    /// Put KaTeX code in the global group.
    /// This means that \def and \newcommand persist in macros across render calls.
    pub global_group: bool,
    pub leq_no: bool,
    pub fleqn: bool,
    /// Whether it should return an error or simply render unsupported commands as text
    /// If the LaTeX is invalid then it will return text with the color of error_color
    pub throw_on_error: bool,
    /// The color an error message would appear as
    pub error_color: RGBA,
    // TODO: We currently clone this in the creation, but it would be better not to
    pub macros: Macros,
    /// Species a minimum thickness for fraction lines, `\sqrt` top lines, `{array}` vertical lines,
    /// `\hline`, `\hdashline`, `\underline`, `\overline` and the borders of `\fbox`, `\boxed`, and
    /// `\fcolorbox`.
    /// The usual value for these items is `0.04`.
    /// Negative values will be ignores.
    pub min_rule_thickness: unit::Em,
    /// In early versions of both KaTeX and MathJax, the `\color` function expected the content to
    /// be a function argument, as in `\color{blue}{hello}`.
    /// In current KaTeX, `\color` is a switch, as in `\color{blue} hello`. This matches LaTeX
    /// behavior.
    /// Setting this to true uses the old behavior.
    pub color_is_text_color: bool,
    /// All user specified sizes, e.g. in `\rule{500em}{500em}`, will be capped to `max_size` ems.
    /// If set to `Infinity` (the default), users can make elements and spaces arbitrarily large.
    pub max_size: unit::Em,
    /// Limit the number of macro expansions to this number, to prevent e.g. infinite macro loops.
    /// If set to `None`, the macro expander will try to fully expand as in LaTeX.
    pub max_expand: Option<u32>,
    /// How strict to be about features which make writing the notation easier but are not
    /// supported by LaTeX itself.
    pub strict: StrictMode,
    // TODO: Allow a function to be used
    /// Whether we should trust the input
    /// This allows things like `\url`, `\includegraphics`, `\htmlClass`, etc.
    pub trust: bool,
}
impl ParserConfig {
    pub fn is_trusted(&self, _command: &str, _url: &str) -> bool {
        // TODO: allow a custom function to be used
        self.trust
    }
}
impl Default for ParserConfig {
    fn default() -> Self {
        ParserConfig {
            display_mode: false,
            global_group: false,
            leq_no: false,
            fleqn: false,
            throw_on_error: true,
            error_color: RGBA::new(0xCC, 0, 0, 0),
            macros: Macros::default(),
            min_rule_thickness: unit::Em(0.04),
            color_is_text_color: false,
            max_size: unit::Em(std::f64::INFINITY),
            max_expand: Some(1000),
            strict: StrictMode::Warn,
            trust: false,
        }
    }
}

/// How strict to be about features that make writing LaTeX convenient but are not actually
/// supported by it.
#[derive(Debug, Clone)]
pub enum StrictMode {
    /// Log errors
    Warn,
    /// Error on the KaTeX statement
    Error,
    // TODO: function callback version
}

#[derive(Clone)]
pub struct Parser<'a, 'f> {
    pub(crate) conf: ParserConfig,
    pub(crate) gullet: MacroExpander<'a, 'f>,
    /// Lookahead token
    next_token: Option<Token<'a>>,
}
impl<'a, 'f> Parser<'a, 'f> {
    pub fn new(input: &'a str, conf: ParserConfig, functions: &'f Functions) -> Parser<'a, 'f> {
        Parser {
            gullet: MacroExpander::new(input, conf.clone(), functions, Mode::Math),
            conf,
            next_token: None,
        }
    }

    pub fn mode(&self) -> Mode {
        self.gullet.mode
    }

    fn expect(&mut self, text: &str, consume: bool) -> Result<(), ParseError> {
        if self.fetch()?.content != text {
            return Err(ParseError::Expected);
        }

        if consume {
            self.consume();
        }

        Ok(())
    }

    fn expect_eof(&mut self) -> Result<(), ParseError> {
        self.expect("EOF", true)
    }

    /// Discards the current lookahead token, returning the value it holds
    pub(crate) fn consume(&mut self) -> Option<Token> {
        self.next_token.take()
    }

    /// Discard any space tokens, fetching the next non-space token.
    pub(crate) fn consume_spaces(&mut self) -> Result<(), ParseError> {
        while self.fetch()?.content == " " {
            self.consume();
        }

        Ok(())
    }

    /// Returns the current lookahead token
    /// Or, if there isn't one,
    ///     such as at the beginning, or if the previous lookahead token was consumed
    /// fetches the next token as the new lookahead token and returns it
    pub(crate) fn fetch(&mut self) -> Result<&Token<'a>, ParseError> {
        if self.next_token.is_none() {
            self.next_token = Some(self.gullet.expand_next_token()?);
        }
        // TODO: is it correct to assume it can't return `None`
        Ok(self.next_token.as_ref().unwrap())
    }

    pub(crate) fn fetch_mut(&mut self) -> Result<&mut Token<'a>, ParseError> {
        if self.next_token.is_none() {
            self.next_token = Some(self.gullet.expand_next_token()?);
        }
        // TODO: is it correct to assume it can't return `None`
        Ok(self.next_token.as_mut().unwrap())
    }

    /// Just returns `next_token`. Guaranteed to exist if the value returned by `fetch`
    /// was `Some` (and no intermediate modifications)  
    /// This primarily exists to get around issues where `fetch` makes Rustc believe that
    /// there is an outstanding mutable borrow.
    pub fn fetch_no_load(&self) -> Option<&Token<'a>> {
        self.next_token.as_ref()
    }

    /// Switch the current mode
    pub(crate) fn switch_mode(&mut self, mode: Mode) {
        self.gullet.switch_mode(mode);
    }

    pub fn dispatch_parse(&mut self) -> Result<Vec<ParseNode>, ParseError> {
        match self.mode() {
            Mode::Math => self.parse::<true>(),
            Mode::Text => self.parse::<false>(),
        }
    }

    /// Main parsing function, which parses an entire input.  
    /// You may wish to use [`dispatch_parse`] instead.  
    pub fn parse<const IS_MATH_MODE: bool>(&mut self) -> Result<Vec<ParseNode>, ParseError> {
        if !self.conf.global_group {
            // Create a group namespace for the math expression
            self.gullet.begin_group();
        }

        // Use old \color behavior (same as LaTeX's \textcolor) if requested.
        if self.conf.color_is_text_color {
            self.gullet.macros.set_back_macro(
                "\\color".to_string(),
                Some(Arc::new(MacroReplace::Text("\\textcolor".to_string()))),
            );
        }

        let err = match self.dispatch_parse_expression(false, None) {
            // If we succeeded, we expect there to be eof at the end
            Ok(result) => match self.expect_eof() {
                Ok(_) => {
                    if !self.conf.global_group {
                        self.gullet.end_group();
                    }

                    self.gullet.end_groups();

                    return Ok(result);
                }
                Err(err) => err,
            },
            Err(err) => err,
        };

        // If we got here then there was an error
        self.gullet.end_groups();

        Err(err)
    }

    /// Fully parse a separate sequence of tokens as a separate job.  
    /// Tokens should be specified in reverse order, as in a macro definition.
    pub(crate) fn sub_parse(
        &mut self,
        tokens: impl Iterator<Item = Token<'a>>,
    ) -> Result<Vec<ParseNode>, ParseError> {
        let old_token = self.next_token.clone();
        self.consume();

        // Run the new job, terminating it with an excess '}'
        self.gullet.push_token(Token::new_text("}"));
        self.gullet.push_tokens(tokens);

        let parse = self.dispatch_parse_expression(false, None)?;
        self.expect("}", true)?;

        self.next_token = old_token;

        Ok(parse)
    }

    /// Dispatches the parse expression logic based on the mode, so that we can have two
    /// versions of the parse expression logic, one for math mode and one for text mode.  
    /// This is a minor perf opt, though it hasn't been checked for significance. However,
    /// it is also likely to be an insignicant slowdown to monomorphize the function.
    pub(crate) fn dispatch_parse_expression(
        &mut self,
        break_on_infix: bool,
        break_on_token_text: Option<BreakToken>,
    ) -> Result<Vec<ParseNode>, ParseError> {
        match self.gullet.mode {
            Mode::Math => self.parse_expression::<true>(break_on_infix, break_on_token_text),
            Mode::Text => self.parse_expression::<false>(break_on_infix, break_on_token_text),
        }
    }

    /// Parses an "expression", which is a list of atoms.  
    /// `break_on_infix`: Whether the parsing should stop when it finds an infix node.  
    /// `break_on_token_text`: The text of the token that the expression should end with
    ///     or `None` if something else should end the expression  
    pub fn parse_expression<const IS_MATH_MODE: bool>(
        &mut self,
        break_on_infix: bool,
        break_on_token_text: Option<BreakToken>,
    ) -> Result<Vec<ParseNode>, ParseError> {
        let mut body = Vec::new();

        loop {
            if IS_MATH_MODE {
                self.consume_spaces()?;
            }

            let token = self.fetch()?.clone();

            if let Some(break_on_token_text) = break_on_token_text.as_ref() {
                if break_on_token_text.matches(&token.content) {
                    break;
                }
            }

            if is_end_of_expression(&token.content) {
                break;
            }

            if break_on_infix {
                if let Some(true) = self
                    .gullet
                    .functions
                    .get(&token.content)
                    .map(|func| func.prop.infix)
                {
                    break;
                }
            }

            let atom = self.parse_atom(break_on_token_text)?;
            if let Some(atom) = atom {
                if matches!(atom, ParseNode::Internal(_)) {
                    continue;
                }

                body.push(atom);
            } else {
                break;
            }
        }

        if self.mode() == Mode::Text {
            self.form_ligatures(&mut body);
        }

        self.handle_infix_nodes(body)
    }

    /// Rewrites infix operators such as `\over` with corresponding commands such as `\frac`.  
    ///
    /// There can only be one infix operator per group. If there is more than one then the
    /// expression is ambiguous. This can be resolved by adding `{}`.
    fn handle_infix_nodes(&mut self, body: Vec<ParseNode>) -> Result<Vec<ParseNode>, ParseError> {
        let mut over_index = None;
        let mut func_name = None;

        for (i, node) in body.iter().enumerate() {
            if let ParseNode::Infix(node) = node {
                if over_index.is_some() {
                    return Err(ParseError::OnlyOneInfixOperatorPerGroup);
                }

                over_index = Some(i);
                func_name = Some(&node.replace_with);
            }
        }

        if let Some(over_index) = over_index {
            // TODO: katex techncailly checks truthiness of this before doing this logic, which would have empty func names count as false
            let func_name = func_name.unwrap();

            let numer_body = &body[0..over_index];
            let denom_body = &body[over_index + 1..];

            let numer_node =
                if numer_body.len() == 1 && matches!(&numer_body[0], ParseNode::OrdGroup(_)) {
                    // TODO: Can we avoid a clone?
                    numer_body[0].clone()
                } else {
                    ParseNode::OrdGroup(OrdGroupNode {
                        body: numer_body.to_owned(),
                        semi_simple: None,
                        info: NodeInfo::new_mode(self.gullet.mode),
                    })
                };

            let denom_node =
                if denom_body.len() == 1 && matches!(&denom_body[0], ParseNode::OrdGroup(_)) {
                    denom_body[0].clone()
                } else {
                    ParseNode::OrdGroup(OrdGroupNode {
                        body: denom_body.to_owned(),
                        semi_simple: None,
                        info: NodeInfo::new_mode(self.gullet.mode),
                    })
                };

            let above_frac = "\\\\abovefrac";
            let node = if func_name == above_frac {
                self.call_function(
                    above_frac,
                    &[numer_node, body[over_index].clone(), denom_node],
                    &[],
                    None,
                    None,
                )?
            } else {
                self.call_function(&func_name, &[numer_node, denom_node], &[], None, None)?
            };

            Ok(vec![node])
        } else {
            Ok(body)
        }
    }

    /// Handle a subscript or superscript
    fn handle_sup_subscript(&mut self, name: &str) -> Result<ParseNode, ParseError> {
        let _symbol_token = self.fetch()?;

        self.consume();
        self.consume_spaces()?;

        self.parse_group(name, None)?
            .ok_or(ParseError::ExpectedGroup)
    }

    /// Converts the textual input of an unsupported command into a text node
    /// contained within a color node whose color is determined by errorColor
    pub(crate) fn format_unsupported_cmd(&self, text: &str) -> UnsupportedCmdParseNode {
        // TODO: We can surely do way better than converting each char to a string

        let text_ord_array = text
            .chars()
            .map(|ch| ch.to_string())
            .map(|text| TextOrdNode {
                text: Cow::Owned(text),
                info: NodeInfo::new_mode(self.gullet.mode),
            })
            .map(ParseNode::TextOrd)
            .collect::<Vec<_>>();

        let text_node = TextNode {
            body: text_ord_array,
            font: None,
            info: NodeInfo::new_mode(self.gullet.mode),
        };

        ColorNode {
            body: vec![ParseNode::Text(text_node)],
            color: Color::RGBA(self.conf.error_color.into_array()),
            info: NodeInfo::new_mode(self.gullet.mode),
        }
    }

    fn parse_atom(
        &mut self,
        break_on_token_text: Option<BreakToken>,
    ) -> Result<Option<ParseNode>, ParseError> {
        // The body of an atom is an implicit group, so that thingsl ike `\left(x\right)^2` work
        // correctly.
        let mut base = self.parse_group("atom", break_on_token_text)?;

        // There are no superscripts or subscripts in text mode
        if self.mode() == Mode::Text {
            return Ok(base);
        }

        let mut superscript = None;
        let mut subscript = None;
        loop {
            self.consume_spaces()?;

            let lex = self.fetch()?;

            let lex_limits = lex.content == "\\limits";
            if lex_limits || lex.content == "\\nolimits" {
                if let Some(base) = &mut base {
                    if let ParseNode::Op(op) = base {
                        op.limits = lex_limits;
                        op.always_handle_sup_sub = Some(true);
                    } else if let ParseNode::OperatorName(op_name) = base {
                        if op_name.always_handle_sup_sub {
                            op_name.limits = lex_limits;
                        }
                    } else {
                        return Err(ParseError::ExpectedLimitControls);
                    }
                } else {
                    return Err(ParseError::ExpectedLimitControls);
                }

                self.consume();
            } else if lex.content == "^" {
                if superscript.is_some() {
                    return Err(ParseError::DoubleSuperscript);
                }

                superscript = Some(self.handle_sup_subscript("superscript")?);
            } else if lex.content == "_" {
                if subscript.is_some() {
                    return Err(ParseError::DoubleSubscript);
                }

                subscript = Some(self.handle_sup_subscript("subscript")?);
            } else if lex.content == "'" {
                // Prime
                if superscript.is_some() {
                    return Err(ParseError::DoubleSuperscript);
                }

                let prime = TextOrdNode {
                    text: Cow::Borrowed("\\prime"),
                    info: NodeInfo::new_mode(self.mode()),
                };

                // TODO: We could slightly optimize the single prime use-case

                self.consume();

                let mut prime_count = 1;

                while self.fetch()?.content == "'" {
                    prime_count += 1;
                    self.consume();
                }

                // If there's a superscript following the primes, then we add that on
                let sup = if self.fetch()?.content == "^" {
                    Some(self.handle_sup_subscript("superscript")?)
                } else {
                    None
                };

                let primes = std::iter::repeat(ParseNode::TextOrd(prime))
                    .take(prime_count)
                    .chain(sup)
                    .collect::<Vec<_>>();

                superscript = Some(ParseNode::OrdGroup(OrdGroupNode {
                    body: primes,
                    semi_simple: None,
                    info: NodeInfo::new_mode(self.mode()),
                }));
            } else if let Some(ch) = unicode::find_sub_map_str(&lex.content) {
                // We found a Unicode subscript or superscript character.
                // We treat these similarly to the unicode-math package.
                // So we render a string of Unicode (sub|super)scripts the same as
                // as a (sub|super)script of regular characters
                let is_sub = unicode::SUB_REGEX.is_match(&lex.content);
                self.consume();

                let mut text = ch.to_string();

                // Continue fetching tokens to fill out the string
                loop {
                    let token = &self.fetch()?.content;
                    let mapped = if let Some(mapped) = unicode::find_sub_map_str(token) {
                        mapped
                    } else {
                        break;
                    };

                    if unicode::SUB_REGEX.is_match(token) != is_sub {
                        break;
                    }

                    self.consume();
                    text.push(mapped);
                }

                let mut parser = Parser::new(&text, self.conf.clone(), self.gullet.functions);
                let body = parser.dispatch_parse()?;

                // TODO: Shouldn't this be checking if super/subscript are already set? Katex
                // doesn't check
                let node = Some(ParseNode::OrdGroup(OrdGroupNode {
                    body,
                    semi_simple: None,
                    info: NodeInfo::new_mode(Mode::Math),
                }));

                if is_sub {
                    subscript = node;
                } else {
                    superscript = node;
                }
            } else {
                // If it wasn't ^, _, or ', stop parsing super/subscripts
                break;
            }
        }

        Ok(if superscript.is_some() || subscript.is_some() {
            Some(ParseNode::SupSub(SupSubNode {
                base: base.map(Box::new),
                // TODO: it seems like super/subscript are maybe only ever ord group nodes?
                // so we could perhaps get rid of these two boxes?
                sup: superscript.map(Box::new),
                sub: subscript.map(Box::new),
                info: NodeInfo::new_mode(self.mode()),
            }))
        } else {
            base
        })
    }

    pub(crate) fn parse_function(
        &mut self,
        break_on_token_text: Option<BreakToken>,
        name: Option<&str>,
    ) -> Result<Option<ParseNode>, ParseError> {
        let token = self.fetch()?.clone();

        let function_name = &token.content;

        let function = if let Some(function) = self.gullet.functions.get(function_name) {
            function
        } else {
            return Ok(None);
        };

        self.consume();

        if name.is_some() && name != Some("atom") && !function.prop.allowed_in_argument {
            return Err(ParseError::FunctionNoArguments);
        } else if self.gullet.mode == Mode::Text && !function.prop.allowed_in_text {
            return Err(ParseError::FunctionUnusableTextMode);
        } else if self.gullet.mode == Mode::Math && !function.prop.allowed_in_math {
            return Err(ParseError::FunctionUnusableMathMode);
        }

        let FunctionArguments { args, opt_args } =
            self.parse_arguments(&function_name, function.clone())?;

        self.call_function(
            function_name,
            &args,
            &opt_args,
            Some(token.clone()),
            break_on_token_text,
        )
        .map(Some)
    }

    /// Call a function handler with a suitable context and arguments
    fn call_function(
        &mut self,
        name: &str,
        args: &[ParseNode],
        opt_args: &[Option<ParseNode>],
        token: Option<Token<'a>>,
        break_on_token_text: Option<BreakToken>,
    ) -> Result<ParseNode, ParseError> {
        let func = self
            .gullet
            .functions
            .get(name)
            .ok_or(ParseError::NoFunctionHandler)?
            .clone();

        let context = FunctionContext {
            func_name: Cow::Borrowed(name),
            parser: self,
            token,
            break_on_token_text,
        };

        Ok((func.handler)(context, args, opt_args))
    }

    fn parse_arguments(
        &mut self,
        func_name: &str,
        spec: Arc<FunctionSpec>,
    ) -> Result<FunctionArguments, ParseError> {
        let total_args = spec.prop.num_args + spec.prop.num_optional_args;
        if total_args == 0 {
            return Ok(FunctionArguments::default());
        }

        let mut args: Vec<ParseNode> = Vec::new();
        let mut opt_args: Vec<Option<ParseNode>> = Vec::new();

        for i in 0..total_args {
            let mut arg_type = spec.prop.arg_types.get(i).copied();

            let is_optional = i < spec.prop.num_optional_args;

            // Note: technically arg_type would be undefined in js if it didn't exist in the
            // argtypes list. Don't think this changes anything
            if (spec.prop.primitive && arg_type.is_none())
                || (spec.prop.typ == ParseNodeType::Sqrt
                    && i == 1
                    && opt_args.get(0).map(Option::as_ref).flatten().is_none())
            {
                arg_type = Some(ArgType::Primitive);
            }

            let arg = self.parse_group_of_type(func_name, arg_type, is_optional)?;
            if is_optional {
                opt_args.push(arg);
            } else if let Some(arg) = arg {
                args.push(arg);
            } else {
                // This shouldn't happen
                return Err(ParseError::NullArgument);
            }
        }

        Ok(FunctionArguments { args, opt_args })
    }

    fn parse_group_of_type(
        &mut self,
        name: &str,
        typ: Option<ArgType>,
        optional: bool,
    ) -> Result<Option<ParseNode>, ParseError> {
        Ok(match typ {
            Some(typ) => self.parse_group_of_arg_type(name, typ, optional)?,
            None => self
                .parse_argument_group(optional, None)?
                .map(ParseNode::OrdGroup),
        })
    }

    fn parse_group_of_arg_type(
        &mut self,
        name: &str,
        typ: ArgType,
        optional: bool,
    ) -> Result<Option<ParseNode>, ParseError> {
        Ok(match typ {
            ArgType::Color => self.parse_color_group(optional)?.map(ParseNode::ColorToken),
            ArgType::Size => self.parse_size_group(optional)?.map(ParseNode::Size),
            ArgType::Url => self.parse_url_group(optional)?.map(ParseNode::Url),
            ArgType::Raw => self
                .parse_string_group(optional)?
                .map(|t| RawNode {
                    string: t.content.into_owned(),
                    info: NodeInfo::new_mode(Mode::Text),
                })
                .map(ParseNode::Raw),
            ArgType::Original => self
                .parse_argument_group(optional, None)?
                .map(ParseNode::OrdGroup),
            ArgType::HBox => self
                .parse_argument_group(optional, Some(Mode::Text))?
                .map(ParseNode::OrdGroup)
                .map(|g| StylingNode {
                    style: Style::Text,
                    body: vec![g],
                    info: NodeInfo::new_mode(Mode::Text),
                })
                .map(ParseNode::Styling),
            ArgType::Primitive => {
                if optional {
                    return Err(ParseError::PrimitiveCantBeOptional);
                }

                Some(
                    self.parse_group(name, None)?
                        .ok_or(ParseError::ExpectedGroup)?,
                )
            }
            ArgType::Mode(mode) => self
                .parse_argument_group(optional, Some(mode))?
                .map(ParseNode::OrdGroup),
        })
    }

    /// Parses a group, essentially returning the string formed by the brace-enclosed tokens plus
    /// some position information.
    fn parse_string_group(&mut self, optional: bool) -> Result<Option<Token<'a>>, ParseError> {
        let arg_token = if let Some(arg_token) = self.gullet.scan_argument(optional)? {
            arg_token
        } else {
            return Ok(None);
        };

        let mut text = String::new();
        loop {
            let token = self.fetch()?;
            if token.is_eof() {
                break;
            }

            text += &token.content;
            self.consume();
        }

        // consume end of the argument
        self.consume();

        Ok(Some(Token::new_owned(text, arg_token.loc)))
    }

    /// Parses a regex-delimited group: the largest sequence of tokens whose concatenated strings
    /// match `regex`. Returns the string formed by the tokens plus some position information.
    fn parse_regex_group(&mut self, regex: &Regex) -> Result<Token<'a>, ParseError> {
        let first_token = self.fetch()?;
        let first_token_loc = first_token.loc.clone();
        let mut last_token_loc = first_token.loc.clone();

        let mut text = String::new();
        loop {
            let next_token = self.fetch()?;
            // TODO: this is less efficient than it could be?
            let test_text = format!("{}{}", text, next_token.content);
            if next_token.is_eof() || !regex.is_match(&test_text) {
                break;
            }

            last_token_loc = next_token.loc.clone();
            text = test_text;

            self.consume();
        }

        if text.is_empty() {
            return Err(ParseError::InvalidRegexMode);
        }

        let loc = SourceLocation::combine(first_token_loc, last_token_loc);
        Ok(Token::new_owned(text, loc))
    }

    /// Parses a color description
    fn parse_color_group(&mut self, optional: bool) -> Result<Option<ColorTokenNode>, ParseError> {
        let res = if let Some(res) = self.parse_string_group(optional)? {
            res
        } else {
            return Ok(None);
        };

        // TODO: At a glance this doesn't support rgba?
        let capture = COLOR_REGEX
            .captures(&res.content)
            .ok_or(ParseError::InvalidColor)?;

        // TODO: better error?
        let color = capture.get(0).ok_or(ParseError::InvalidColor)?;
        let color = color.as_str();

        // TODO: This is very ugly
        let color = if color.len() == 6 {
            if let Ok(color) = parse_rgb(color) {
                Color::RGB(color)
            } else {
                Color::Named(color.to_string().into())
            }
        } else if color.len() == 7 && color.starts_with('#') {
            if let Ok(color) = parse_rgb(&color[1..]) {
                Color::RGB(color)
            } else {
                // Probably bad color
                Color::Named(color.to_string().into())
            }
        } else if color.len() == 4 && color.starts_with('#') {
            if let Ok(color) = parse_rgb_3(&color[1..]) {
                Color::RGB(color)
            } else {
                // Probably bad color
                Color::Named(color.to_string().into())
            }
        } else if color.len() == 9 && color.starts_with('#') {
            if let Ok(color) = parse_rgba(&color[1..]) {
                Color::RGBA(color)
            } else {
                // Probably bad color
                Color::Named(color.to_string().into())
            }
            // TODO: short version of rgba?
        } else {
            Color::Named(color.to_string().into())
        };

        Ok(Some(ColorTokenNode {
            color,
            info: NodeInfo::new_mode(self.mode()),
        }))
    }

    fn parse_size_group(&mut self, optional: bool) -> Result<Option<SizeNode>, ParseError> {
        let mut is_blank = false;

        // don't expand before parseStringGroup
        self.gullet.consume_spaces()?;

        let res = if !optional && self.gullet.future()?.content != "{" {
            Some(self.parse_regex_group(&SIZE_GROUP_REGEX)?)
        } else {
            self.parse_string_group(optional)?
        };

        let mut res = if let Some(res) = res {
            res
        } else {
            return Ok(None);
        };

        if !optional && res.content.is_empty() {
            // Since we have tested for what is non-optional, this won't affect \kern,
            // \hspace, etc. It will capture the mandatory arguments to \genfrac and \above

            // enable \above{}
            res.content = Cow::Borrowed("0pt");
            // for \genfrac
            is_blank = true;
        }

        let captures = SIZE_REGEX
            .captures(&res.content)
            .ok_or(ParseError::InvalidSize)?;

        let sign = captures.get(1);
        let magnitude = captures.get(2);
        let num = if let Some((sign, magnitude)) = sign.zip(magnitude) {
            let sign = sign.as_str();
            let magnitude = magnitude.as_str();
            // TODO: option to error on this
            let magnitude = magnitude.parse::<f64>().unwrap_or(std::f64::NAN);
            if sign == "-" {
                -magnitude
            } else {
                magnitude
            }
        } else {
            // KaTeX doesn't check for if it is a valid number
            std::f64::NAN
        };
        let unit = captures.get(3).ok_or(ParseError::InvalidUnit)?.as_str();

        let measure = Measurement::from_unit(num, unit).ok_or(ParseError::InvalidUnit)?;

        Ok(Some(SizeNode {
            value: measure,
            is_blank,
            info: NodeInfo::new_mode(self.mode()),
        }))
    }

    /// Parses an URL, checking escapes letters and allowed protocols
    fn parse_url_group(&mut self, optional: bool) -> Result<Option<UrlNode>, ParseError> {
        self.gullet.lexer.set_catcode('%', CategoryCode::Active);
        self.gullet.lexer.set_catcode('~', CategoryCode::Other);
        let res = self.parse_string_group(optional)?;
        // Reset the catcode redefinitions
        self.gullet.lexer.remove_catcode('%');
        self.gullet.lexer.remove_catcode('~');

        let res = if let Some(res) = res {
            res
        } else {
            return Ok(None);
        };

        let url = URL_REGEX.replace_all(&res.content, "$1");

        Ok(Some(UrlNode {
            url: url.into_owned(),
            info: NodeInfo::new_mode(self.mode()),
        }))
    }

    fn parse_argument_group(
        &mut self,
        optional: bool,
        mode: Option<Mode>,
    ) -> Result<Option<OrdGroupNode>, ParseError> {
        let arg_token = if let Some(arg_token) = self.gullet.scan_argument(optional)? {
            arg_token
        } else {
            return Ok(None);
        };

        let outer_mode = self.mode();
        if let Some(mode) = mode {
            self.switch_mode(mode);
        }

        self.gullet.begin_group();

        let expression = self.dispatch_parse_expression(false, Some(BreakToken::EOF))?;
        self.expect_eof()?;
        self.gullet.end_group();

        let ord = OrdGroupNode {
            body: expression,
            semi_simple: None,
            info: NodeInfo {
                mode: self.mode(),
                loc: arg_token.loc,
            },
        };

        if mode.is_some() {
            self.switch_mode(outer_mode);
        }

        Ok(Some(ord))
    }

    /// Parses an ordinary group, which is either a single nucleus (like "x")
    /// or an expression in brace (like "{x+y") or an implicit group, a group that starts
    /// at the current position, and ends right before a higher explicit group ends, or at EOF
    fn parse_group(
        &mut self,
        name: &str,
        break_on_token_text: Option<BreakToken>,
    ) -> Result<Option<ParseNode>, ParseError> {
        let first_token = self.fetch()?.clone();

        let is_left_curly = first_token.content == "{";
        let is_begin_group = first_token.content == "\\begingroup";
        if is_left_curly || is_begin_group {
            let first_token_loc = first_token.loc.clone();
            self.consume();

            // Get the text that signifies the end of this group
            let group_end = if is_left_curly {
                BreakToken::RightCurlyBracket
            } else {
                BreakToken::EndGroup
            };

            self.gullet.begin_group();

            // If we get a brace, parse an expression
            let expression = self.dispatch_parse_expression(false, Some(group_end))?;

            let last_token = self.fetch()?;
            let loc = SourceLocation::combine(first_token_loc, last_token.loc.clone());

            // Ensure we've got the matching closing
            self.expect(group_end.as_str(), true)?;

            self.gullet.end_group();

            Ok(Some(ParseNode::OrdGroup(OrdGroupNode {
                body: expression,
                semi_simple: is_begin_group.then(|| true),
                info: NodeInfo {
                    mode: self.mode(),
                    loc,
                },
            })))
        } else {
            let res = self.parse_function(break_on_token_text, Some(name))?;
            let res = if let Some(res) = res {
                Some(res)
            } else {
                self.parse_symbol()?
            };

            if res.is_none()
                && first_token.content.starts_with('\\')
                && !is_implicit_command(&first_token.content)
            {
                if self.conf.throw_on_error {
                    return Err(ParseError::UndefinedControlSequence(
                        first_token.content.to_string(),
                    ));
                }

                let res = self.format_unsupported_cmd(&first_token.content);
                self.consume();
                Ok(Some(ParseNode::Color(res)))
            } else {
                Ok(res)
            }
        }
    }

    /// Form ligature-like combinations of characters for text mode.  
    /// This includes inputs like "--", "---", "\`\`" and "''"  
    /// The result will simply replace multiple textord nodes with a single character
    /// in each value by a single textord node having multiple characters in its value. The
    /// representation is till ASCII source.  
    /// The group will be modified in place.  
    fn form_ligatures(&mut self, group: &mut Vec<ParseNode>) {
        if group.is_empty() {
            return;
        }

        // TODO: setting for whether to form ligatures or not

        let mut n = group.len() - 1;
        let mut i = 0;
        loop {
            if i >= n {
                break;
            }

            let a = &group[i];
            let a_loc = a.loc();
            // TODO: It would be nice to avoid this alloc. We have to do it because we start
            // messing with `group` and it would literally be removed
            let text = a.text().map(|x| x.to_string());
            let text = text.as_deref();
            if text == Some("-") && group.get(i + 1).and_then(ParseNode::text) == Some("-") {
                if i + 1 < n && group.get(i + 2).and_then(ParseNode::text) == Some("-") {
                    group.remove(i);
                    group.remove(i);
                    // Instead of doing the third remove, we just replace it, which is probably
                    // slightly better
                    // group.remove(i);
                    group[i] = ParseNode::TextOrd(TextOrdNode {
                        text: Cow::Borrowed("---"),
                        info: NodeInfo {
                            mode: Mode::Text,
                            loc: SourceLocation::combine(
                                a_loc.clone(),
                                group.get(i + 2).and_then(ParseNode::loc),
                            ),
                        },
                    });
                    n = n.saturating_sub(2);
                } else {
                    group.remove(i);
                    group[i] = ParseNode::TextOrd(TextOrdNode {
                        text: Cow::Borrowed("--"),
                        info: NodeInfo {
                            mode: Mode::Text,
                            loc: SourceLocation::combine(
                                a_loc.clone(),
                                group.get(i + 1).and_then(ParseNode::loc),
                            ),
                        },
                    });
                    n = n.saturating_sub(1);
                }
            }

            // If it is a ' or ` and then then next entry is the same
            if (text == Some("'") || text == Some("`"))
                && group.get(i + 1).and_then(ParseNode::text) == text
            {
                let text = text.unwrap();
                group.remove(i);
                group[i] = ParseNode::TextOrd(TextOrdNode {
                    text: format!("{text}{text}").into(),
                    info: NodeInfo {
                        mode: Mode::Text,
                        loc: SourceLocation::combine(
                            a_loc,
                            group.get(i + 1).and_then(ParseNode::loc),
                        ),
                    },
                })
            }

            i += 1;
        }
    }

    /// Parse a single symbol out of the string. Here we handle single character smybols and special
    /// functions like \verb
    fn parse_symbol(&mut self) -> Result<Option<ParseNode>, ParseError> {
        let nucleus = self.fetch()?.clone();
        let nucleus_loc = nucleus.loc.clone();

        if SYMBOL_VERB_REGEX.is_match(&nucleus.content) {
            self.consume();

            let arg = &nucleus.content[5..];
            let star = arg.starts_with('*');
            let arg = if star { &arg[1..] } else { arg };

            // Lexer's token regex is constructed to always have matching first/last characters
            // TODO: Do the check the TS file has

            // Remove first and last character
            let arg = &arg[1..arg.len() - 1];

            return Ok(Some(ParseNode::Verb(VerbNode {
                body: arg.to_string().into(),
                star,
                info: NodeInfo::new_mode(Mode::Text),
            })));
        }

        let mut text = nucleus.content;
        // We need an &str for symbols, but a char for unicode symbols..
        if let Some(first) = first_ch_str(&text) {
            let first_ch = first.chars().next().unwrap();
            // If it is in the unicode symbols but not in the defined symbols
            if let Some(symbol) = unicode::SYMBOLS.get(&first_ch) {
                if symbols::SYMBOLS.get(self.mode(), first).is_none() {
                    // TODO: warn/error if strict and the mode is math
                    let sub = &text[1..];
                    text = Cow::Owned(format!("{symbol}{sub}"));
                }
            }
        }

        // Strip off any combining characters
        let first_match = COMBINING_DIACRITICAL_MARKS_END_REGEX
            .captures(&text)
            .and_then(|c| c.get(0));
        let transforms = if let Some(first) = &first_match {
            // KaTeX does this near the end of this func,
            // but we do it here to minimize clones and the like

            let mut transforms = Vec::new();
            for accent_ch in first.as_str().chars() {
                let accent = unicode::get_accent(accent_ch).ok_or(ParseError::UnknownAccent)?;

                let command = accent.get_mode(self.mode());
                // TODO: Katex checks if command is valid but they always should get one? Am I
                // missing something?

                transforms.push(command);
            }

            // This is what is normally done
            let start = first.start();
            text = match text {
                Cow::Borrowed(bor) => Cow::Borrowed(&bor[0..start]),
                Cow::Owned(mut own) => {
                    own.truncate(start);
                    Cow::Owned(own)
                }
            };

            if text == "i" {
                // dotless i
                text = Cow::Borrowed("\u{0131}");
            } else if text == "j" {
                // dotless j
                text = Cow::Borrowed("\u{0237}");
            }

            Some(transforms)
        } else {
            None
        };

        let symbol = if let Some(symbol) = symbols::SYMBOLS.get(self.mode(), &text) {
            // TODO: warn/error if strict and mode is math and extra latin contains text

            let group = symbol.group;

            match group {
                Group::Atom(atom_g) => ParseNode::Atom(AtomNode {
                    family: atom_g,
                    text: text.into_owned(),
                    info: NodeInfo::new_mode(self.mode()),
                }),
                Group::NonAtom(n_atom) => n_atom.make_parse_node(
                    NodeInfo {
                        mode: self.mode(),
                        loc: nucleus_loc.clone(),
                    },
                    text.into_owned(),
                ),
            }
            // TODO: is this 0x80 check mimicking katex right?
        } else if let Some(true) = text.chars().next().map(|x| x as u32 >= 0x80) {
            // TODO: do warnings on bad characters

            ParseNode::TextOrd(TextOrdNode {
                text: text.into_owned().into(),
                info: NodeInfo {
                    mode: Mode::Text,
                    loc: SourceLocation::combine(nucleus_loc.clone(), None),
                },
            })
        } else {
            return Ok(None);
        };

        self.consume();

        if let Some(transforms) = transforms {
            let mut symbol = symbol;
            let node_info = NodeInfo {
                mode: self.mode(),
                loc: SourceLocation::combine(nucleus_loc, None),
            };
            for label in transforms {
                symbol = ParseNode::Accent(AccentNode {
                    label: Cow::Borrowed(label),
                    is_stretchy: Some(false),
                    is_shifty: Some(true),
                    base: Box::new(symbol),
                    info: node_info.clone(),
                });
            }

            Ok(Some(symbol))
        } else {
            Ok(Some(symbol))
        }
    }
}

fn is_end_of_expression(text: &str) -> bool {
    text == "}" || text == "\\endgroup" || text == "\\end" || text == "\\right" || text == "&"
}

#[derive(Default)]
pub struct FunctionArguments {
    pub args: Vec<ParseNode>,
    pub opt_args: Vec<Option<ParseNode>>,
}
