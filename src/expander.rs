use std::borrow::{Borrow, Cow};

use crate::{
    builtin_macros::BUILTIN_MACROS,
    functions::Functions,
    lexer::{CategoryCode, Lexer, LexerConf, Token},
    macr::{MacroArg, MacroExpansion, MacroReplace},
    namespace::Namespace,
    parser::{ParseError, ParserConfig},
    symbols,
    util::SourceLocation,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    Math,
    Text,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BreakToken {
    // Note that these comments do not escape backslashes
    /// "]"
    RightSquareBracket,
    /// "}"
    RightCurlyBracket,
    /// "\endgroup"
    EndGroup,
    /// "$"
    Dollar,
    /// "\)"
    BackslashRightParen,
    /// "\\"
    DoubleBackslash,
    /// "\end"
    End,
    EOF,
}
impl BreakToken {
    pub fn as_str(&self) -> &str {
        match self {
            BreakToken::RightSquareBracket => "]",
            BreakToken::RightCurlyBracket => "}",
            BreakToken::EndGroup => "\\endgroup",
            BreakToken::Dollar => "$",
            BreakToken::BackslashRightParen => "\\)",
            BreakToken::DoubleBackslash => "\\\\",
            BreakToken::End => "\\end",
            BreakToken::EOF => "EOF",
        }
    }

    /// Note that `None` is assumed to mean EOF
    pub fn matches(&self, t: &str) -> bool {
        self.as_str() == t
    }
}

#[derive(Debug, Clone)]
pub enum TokenList<'a> {
    Single(Token<'a>),
    List(Vec<Token<'a>>),
}

pub(crate) const IMPLICIT_COMMANDS: &'static [&'static str] = &["^", "_", "\\limits", "\\nolimits"];
pub(crate) fn is_implicit_command(text: &str) -> bool {
    IMPLICIT_COMMANDS.iter().any(|cmd| *cmd == text)
}

#[derive(Clone)]
pub struct MacroExpander<'a, 'f> {
    pub(crate) lexer: Lexer<'a>,
    pub(crate) macros: Namespace,
    pub functions: &'f Functions,
    pub(crate) mode: Mode,
    conf: ParserConfig,
    stack: Vec<Token<'a>>,

    /// The number of expansions we've done. Used to make sure that we don't exceed any limits.
    expansion_count: u32,
}
impl<'a, 'f> MacroExpander<'a, 'f> {
    pub fn new(
        input: &'a str,
        conf: ParserConfig,
        functions: &'f Functions,
        mode: Mode,
    ) -> MacroExpander<'a, 'f> {
        MacroExpander {
            lexer: Lexer::new(input, LexerConf {}),
            macros: Namespace::new(BUILTIN_MACROS.clone(), conf.macros.clone()),
            functions,
            conf,
            mode,
            stack: Vec::new(),

            expansion_count: 0,
        }
    }

    /// Swap out the input, which requires 'remaking' it to change the lifetime
    pub fn refeed(self, input: &str) -> MacroExpander<'_, 'f> {
        MacroExpander {
            lexer: Lexer::new(input, self.lexer.conf),
            macros: self.macros,
            functions: self.functions,
            conf: self.conf,
            mode: self.mode,
            // TODO: Should we be keeping the stack? KaTeX's feed function only swaps the lexer..
            // TODO: Theoretically, we could keep the existing allocation?
            stack: Vec::new(),

            expansion_count: 0,
        }
    }

    pub fn switch_mode(&mut self, new_mode: Mode) {
        self.mode = new_mode;
    }

    pub fn begin_group(&mut self) {
        self.macros.begin_group();
    }

    pub fn end_group(&mut self) {
        self.macros.end_group();
    }

    pub fn end_groups(&mut self) {
        self.macros.end_groups();
    }

    /// Returns the topmost token on the stack, without expanding it.
    /// If there are no entries on the stack, it lexes one out
    /// Returns `None` if lexing would result in EOF
    pub fn future(&mut self) -> Result<&Token<'a>, ParseError> {
        if self.stack.is_empty() {
            let token = self.lexer.lex()?;
            self.stack.push(token);
        }

        // TODO: Should this be panicking? JS would just return undefined.
        Ok(self.stack.last().unwrap())
    }

    /// Remove and return the next unexpanded token
    pub(crate) fn pop_token(&mut self) -> Result<Token<'a>, ParseError> {
        self.future()?;
        // TODO: Should this be panicking? JS would just return undefined.
        Ok(self.stack.pop().unwrap())
    }

    pub(crate) fn push_token(&mut self, token: Token<'a>) {
        self.stack.push(token);
    }

    pub(crate) fn push_tokens(&mut self, tokens: impl Iterator<Item = Token<'a>>) {
        for token in tokens {
            self.stack.push(token);
        }
    }

    /// Consume all the following space tokens
    pub fn consume_spaces(&mut self) -> Result<(), ParseError> {
        while self.future()?.content == " " {
            self.stack.pop();
        }

        Ok(())
    }

    /// Find a macro argument without expanding tokens and append the array of tokens
    /// to the token stack. Uses [`Token`] as a container for the result
    pub fn scan_argument(&mut self, is_optional: bool) -> Result<Option<Token<'a>>, ParseError> {
        let (start, end, tokens) = if is_optional {
            // \@ifnextchar gobbles any space following it
            self.consume_spaces()?;
            if self.future()?.content != "[" {
                return Ok(None);
            }

            let start = self.pop_token()?;
            let arg = self.consume_arg_delim(&["]"])?;

            (start, arg.end, arg.tokens)
        } else {
            let arg = self.consume_arg()?;
            (arg.start, arg.end, arg.tokens)
        };

        // indicate thend of an arguments
        self.push_token(Token::eof(end.loc.clone()));

        self.push_tokens(tokens.into_iter());

        let loc = if let Some((start, end)) = start.loc.zip(end.loc) {
            Some(SourceLocation(start.0.start..end.0.end))
        } else {
            None
        };

        Ok(Some(Token::new_opt("", loc)))
    }

    pub fn consume_arg(&mut self) -> Result<MacroArg<'a>, ParseError> {
        self.consume_arg_delim::<String>(&[])
    }

    /// Consume an argument from the token stream, and return the resulting array of tokens
    /// and the start/end token
    pub fn consume_arg_delim<T: AsRef<str>>(
        &mut self,
        delimiters: &[T],
    ) -> Result<MacroArg<'a>, ParseError> {
        let mut tokens = Vec::new();

        let is_delimited = !delimiters.is_empty();

        if !is_delimited {
            // Ignore spaces between arguments
            self.consume_spaces()?;
        }

        let start = self.future()?.clone();

        let mut depth: usize = 0;
        let mut matc: usize = 0;
        let mut tok;
        loop {
            tok = self.pop_token()?;

            // TODO: can we do better than a clone
            tokens.push(tok.clone());

            // Keep track of the bracket depth
            if tok.content == "{" {
                depth += 1;
            } else if tok.content == "}" {
                if let Some(sub_depth) = depth.checked_sub(1) {
                    depth = sub_depth;
                } else {
                    // TODO: extra } error
                    panic!();
                }
            } else if tok.is_eof() {
                return Err(ParseError::UnexpectedEOF);
            }

            if is_delimited {
                let delim_match = delimiters.get(matc).map(AsRef::as_ref);
                if (depth == 0 || (depth == 1 && delim_match == Some("{")))
                    && Some(tok.content.borrow()) == delim_match
                {
                    //
                    matc += 1;
                    if matc == delimiters.len() {
                        // Don't include the delimiters in tokens
                        // TODO: splice it!
                        break;
                    }
                } else {
                    matc = 0;
                }
            }

            if !(depth != 0 || is_delimited) {
                break;
            }
        }

        let is_start_bracket = start.content == "{";
        let is_end_bracket = tokens.last().map(|x| x.content == "}").unwrap_or(false);

        if is_start_bracket && is_end_bracket {
            tokens.pop();
            tokens.remove(0);
        }

        // TODO: I can do better than this, probably. Though probably not that much actual
        // processing done.
        tokens.reverse();

        Ok(MacroArg {
            tokens,
            start,
            // There has to be a token, since we would error if the first token we got was an EOF
            end: tok,
        })
    }

    /// Consume the specified number of arguments from the token stream
    /// and return the resulting array of arguments
    pub fn consume_args(&mut self, arg_num: usize) -> Result<Vec<Vec<Token<'a>>>, ParseError> {
        self.consume_args_delim::<String>(arg_num, &[])
    }

    // TODO: It would be nice to make the delimiters more generic
    /// Consume the specified number of (delimited) arguments from the token stream
    /// and return the resulting array of arguments
    pub fn consume_args_delim<T: AsRef<str>>(
        &mut self,
        arg_num: usize,
        delimiters: &[Vec<T>],
    ) -> Result<Vec<Vec<Token<'a>>>, ParseError> {
        if !delimiters.is_empty() {
            if delimiters.len() != arg_num + 1 {
                return Err(ParseError::MismatchDelimitersArgsLength);
            }

            let delimiters = &delimiters[0];
            for delim in delimiters {
                let token = self.pop_token()?;
                if token.content != delim.as_ref() {
                    return Err(ParseError::MismatchMacroDefinition);
                }
            }
        }

        let mut args = Vec::new();

        for i in 0..arg_num {
            let delims = if !delimiters.is_empty() {
                delimiters[i + 1].as_slice()
            } else {
                &[]
            };
            let arg = self.consume_arg_delim(delims)?;
            args.push(arg.tokens);
        }

        Ok(args)
    }

    pub fn consume_args_n<const N: usize>(&mut self) -> Result<[Vec<Token<'a>>; N], ParseError> {
        self.consume_args_delim_n::<N, String>(&[])
    }

    pub fn consume_args_delim_n<const N: usize, T: AsRef<str>>(
        &mut self,
        delimiters: &[Vec<T>],
    ) -> Result<[Vec<Token<'a>>; N], ParseError> {
        if !delimiters.is_empty() {
            if delimiters.len() != N + 1 {
                return Err(ParseError::MismatchDelimitersArgsLength);
            }

            let delimiters = &delimiters[0];
            for delim in delimiters {
                let token = self.pop_token()?;
                if token.content != delim.as_ref() {
                    return Err(ParseError::MismatchMacroDefinition);
                }
            }
        }

        let mut args: [Vec<Token>; N] = std::array::from_fn(|_| Vec::new());
        for i in 0..N {
            let delims = if !delimiters.is_empty() {
                delimiters[i + 1].as_slice()
            } else {
                &[]
            };
            let arg = self.consume_arg_delim(delims)?;
            args[i] = arg.tokens;
        }

        Ok(args)
    }

    /// Expand the next token only once if possible.
    ///
    /// If the token is expanded, the resulting tokens will be pushed onto the stack in reverse
    /// order and will be returned, also in reverse order.
    ///
    /// If not, the next token will be returned without removing it from the stack.
    ///
    /// In either case, the next token will be on top of the stack or the stack will be empty.
    /// Note that this returns `None` on EOF, and so it won't put anything on the stack for that.
    ///
    /// If `expandable_only`, only expandable tokens are expanded and an undefiend control sequence
    /// results in an error.
    pub(crate) fn expand_once(
        &mut self,
        expandable_only: bool,
    ) -> Result<TokenList<'_>, ParseError> {
        let top_token = self.pop_token()?;
        let _macro_name = top_token.content.clone();

        let expansion = if !top_token.no_expand {
            self.get_expansion(&top_token.content)?
        } else {
            None
        };

        // Ugly conditional
        let cannot_expand = expansion.is_none()
            || (expandable_only && expansion.as_ref().map(|e| e.unexpandable).unwrap_or(false));

        if cannot_expand {
            let name = &top_token.content;
            if expandable_only
                && expansion.is_none()
                && name.starts_with('\\')
                && !self.is_defined(name)
            {
                return Err(ParseError::UndefinedControlSequence(name.to_string()));
            }

            self.push_token(top_token.clone());
            return Ok(TokenList::Single(top_token));
        }

        let expansion = expansion.unwrap();

        self.expansion_count += 1;

        if self.conf.max_expand.is_some() && self.expansion_count > self.conf.max_expand.unwrap() {
            return Err(ParseError::TooManyExpansions);
        }

        let mut tokens = expansion.tokens;

        let num_args = expansion.num_args as usize;
        let delimiters = expansion.delimiters.as_deref().unwrap_or(&[]);

        let args = self.consume_args_delim(num_args, delimiters)?;
        if num_args != 0 {
            // paste arguments in place of placeholders (tokens are stored in reverse order)
            let mut i = tokens.len();
            while i > 0 {
                i -= 1;
                if tokens[i].content == "#" {
                    if i == 0 {
                        return Err(ParseError::IncompletePlaceholder);
                    }

                    let prev_content = tokens[i - 1].content.clone();
                    if prev_content == "#" {
                        // ## -> literal #
                        tokens.remove(i);
                        if i > 0 {
                            i -= 1; // skip the remaining '#'
                        }
                        continue;
                    }

                    if prev_content.len() == 1 {
                        if let Some(ch) = prev_content.chars().next() {
                            if ch != '0' {
                                if let Some(arg_index) = ch.to_digit(10) {
                                    let arg_index = (arg_index - 1) as usize;

                                    let i_args = args
                                        .get(arg_index)
                                        .ok_or(ParseError::InvalidArgumentNumber)?;

                                    tokens.splice(i - 1..=i, i_args.iter().cloned());

                                    continue;
                                }
                            }
                        }
                    }

                    return Err(ParseError::InvalidArgumentNumber);
                }
            }
        }

        self.push_tokens(tokens.iter().cloned());

        Ok(TokenList::List(tokens))
    }

    /// Expand the next token only once (if possible), and return the resulting top token
    /// on the stack (without removing anything from the stack).
    /// Similar in behavior to TeX's `\expandafter\futurelet`
    /// Equivalent to `expand_once(false)` followed by `future()`
    pub fn expand_after_future(&mut self) -> Result<&Token<'a>, ParseError> {
        self.expand_once(false)?;
        self.future()
    }

    /// Recursively expand first token, then return first non-expandable token
    pub fn expand_next_token(&mut self) -> Result<Token<'a>, ParseError> {
        loop {
            let expanded = self.expand_once(false)?;
            match expanded {
                TokenList::Single(mut token) => {
                    if token.treat_as_relax {
                        token.content = Cow::Borrowed("\\relax");
                    }

                    return Ok(self.stack.pop().unwrap());
                }
                TokenList::List(_) => {}
            }
        }
    }

    /// Fully expand the given macro name and return the resulting list of tokens,
    /// or return `None` if no such macro is defined.
    pub fn expand_macro(&mut self, name: &'a str) -> Result<Option<Vec<Token<'a>>>, ParseError> {
        if self.macros.contains_back_macro(name) {
            // TODO: better nonexistent source loc
            let token = Token::new(name, 0..0);
            Ok(Some(self.expand_tokens([token].into_iter())?))
        } else {
            Ok(None)
        }
    }

    pub fn expand_tokens(
        &mut self,
        tokens: impl Iterator<Item = Token<'a>>,
    ) -> Result<Vec<Token<'a>>, ParseError> {
        let mut output = Vec::new();
        let old_stack_len = self.stack.len();

        for token in tokens {
            self.push_token(token);
        }

        while self.stack.len() > old_stack_len {
            // We only want to expand the expandable tokens
            let expanded = self.expand_once(true)?;

            // expand_once returns a single token iff it is fully expanded
            match expanded {
                TokenList::Single(mut token) => {
                    if token.treat_as_relax {
                        token.no_expand = false;
                        token.treat_as_relax = false;
                    }

                    output.push(self.stack.pop().unwrap());
                }
                TokenList::List(_) => {}
            }
        }

        Ok(output)
    }

    /// Fully expand the given macro name and return the result as a string,
    /// or return `None` if no such macro is defined.
    pub fn expand_macro_as_text(&mut self, name: &'a str) -> Result<Option<String>, ParseError> {
        let tokens = if let Some(tokens) = self.expand_macro(name)? {
            tokens
        } else {
            return Ok(None);
        };
        let content = tokens.into_iter().map(|tok| tok.content);
        Ok(Some(content.collect::<String>()))
    }

    /// Returns the expanded macro as a reversed list of tokens and a macro argument count
    /// Or it returns `None` if there is no such macro.
    fn get_expansion(&mut self, name: &str) -> Result<Option<MacroExpansion<'a>>, ParseError> {
        // If it is a single character and it has an associated category code that is not 13
        // (active character) then don't expand it
        if name.len() == 1 {
            let first = name.chars().next().unwrap();
            if self.lexer.catcode(first) != Some(CategoryCode::Active) {
                return Ok(None);
            }
        }

        // Is this skipping letter macro expansion?
        // TODO: This can be a function!
        let definition = if let Some(def) = self.macros.get_back_macro(name) {
            def.clone()
        } else {
            return Ok(None);
        };

        let val = definition.exec(self)?;
        let expansion = val.into_expansion(&self.lexer.conf)?;
        Ok(Some(expansion))
    }

    /// Checks whether a command is currently 'defined' (it has some functionality)
    /// meaning that it's a macro (in the current group), a function, a symbol, or
    /// or one of the special commands listed in `implicit_commands `
    pub(crate) fn is_defined(&self, name: &str) -> bool {
        self.macros.contains_back_macro(name)
            || self.functions.get(name).is_some()
            || symbols::SYMBOLS.contains_key(Mode::Math, name)
            || symbols::SYMBOLS.contains_key(Mode::Text, name)
            || is_implicit_command(name)
    }

    /// Determine whether a command by the given name is expandable
    pub(crate) fn is_expandable(&self, name: &str) -> bool {
        if let Some(macr) = self.macros.get_back_macro(name) {
            // if it is a string or a function or !unexpandable
            match macr.as_ref() {
                MacroReplace::Text(_) | MacroReplace::Func(_) => true,
                MacroReplace::Expansion(exp) => !exp.unexpandable,
            }
        } else if let Some(func) = self.functions.get(name) {
            !func.prop.primitive
        } else {
            false
        }
    }
}
