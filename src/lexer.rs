#![allow(clippy::upper_case_acronyms)]

use std::{borrow::Cow, collections::HashMap};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{parser::ParseError, util::SourceLocation};

static TOKEN_REGEX: Lazy<Regex> = Lazy::new(|| {
    // TODO: This does not include all of the parts that the KaTeX regex does
    // This does not include the verb parts and skips some of the maybe utf16 unicode
    const REGEX_TEXT: &str =
        "([ \\r\\n\\t]+)|\\\\(\\n|[ \\r\\t]+\\n?)[ \\r\\t]*|([!-\\[\\]-\\u2027\\u202A-\\uD7FF\\uF900-\\uFFFF][\\u0300-\\u036f]*|(\\\\[a-zA-Z@]+)[ \\r\\n\\t]*|\\\\.)";
    // KaTeX match index to our index mapping
    // [1] => [1] regular whitespace
    // [2] => [2] backslash whitespace, \whitespace
    // [3] 'anything else'
    //   [4] [5] => nonexistent
    // [6] => [4] backslash followed by word, excluding any trailing whitespace
    Regex::new(REGEX_TEXT).unwrap()
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum CategoryCode {
    Comment = 14,
    Active = 13,
    Other = 12,
}

#[derive(Debug, Clone)]
pub struct LexerConf {}

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    input: &'a str,
    pub(crate) conf: LexerConf,
    // TODO: this can theoretically be strings and custom codes
    catcodes: HashMap<char, CategoryCode>,
    pos: usize,
}
impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, conf: LexerConf) -> Lexer<'a> {
        Lexer {
            input,
            conf,
            catcodes: HashMap::new(),
            pos: 0,
        }
    }

    /// Lex a single token
    pub fn lex(&mut self) -> Result<Token<'a>, ParseError> {
        if self.pos >= self.input.len() {
            // We shouldn't end up in a position where we're outside 1 out of bounds typically but
            // a loose check will avoid crashing
            debug_assert_eq!(self.pos, self.input.len());
            return Ok(Token::eof(Some(self.pos..self.pos)));
        };

        let initial_pos = self.pos;
        let input = &self.input[self.pos..];
        let text = if let Some(capture) = TOKEN_REGEX.captures(input) {
            let (text, end) = if let Some(mac) = capture.get(4) {
                // backslash macro
                (mac.as_str(), mac.end())
            } else if let Some(mac) = capture.get(3) {
                // other things
                (mac.as_str(), mac.end())
            } else if let Some(mac) = capture.get(2) {
                // TODO: is this correct?
                ("\\ ", mac.end())
            } else {
                // TODO: is this correct?
                (" ", ' '.len_utf8())
            };

            self.pos += end;

            // Ignore everything after on the same line as comment character
            if text.len() == 1 {
                if let Some(first) = text.chars().next() {
                    if self.is_comment_character(first) {
                        // TODO: ensure pos is valid
                        let dest = &self.input[self.pos..];
                        if let Some(nl_index) = dest.find('\n') {
                            self.pos += nl_index + '\n'.len_utf8();
                        } else {
                            // TODO: report nonstrict error
                            // eof
                            self.pos = self.input.len();
                        }
                        return self.lex();
                    }
                }
            }

            text
        } else {
            return Err(ParseError::UnexpectedChar(
                self.pos,
                input.chars().nth(self.pos).unwrap(),
            ));
        };

        Ok(Token::new(text, SourceLocation(initial_pos..self.pos)))
    }

    fn is_comment_character(&self, ch: char) -> bool {
        self.catcode(ch) == Some(CategoryCode::Comment)
    }

    pub(crate) fn set_catcode(&mut self, ch: char, code: CategoryCode) {
        self.catcodes.insert(ch, code);
    }

    pub(crate) fn remove_catcode(&mut self, ch: char) {
        self.catcodes.remove(&ch);
    }

    pub(crate) fn catcode(&self, ch: char) -> Option<CategoryCode> {
        if let Some(code) = self.catcodes.get(&ch) {
            return Some(*code);
        }
        Some(match ch {
            '%' => CategoryCode::Comment,
            '~' => CategoryCode::Active,
            _ => return None,
        })
    }
}

/// Note: We don't have any special EOF token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'a> {
    pub content: Cow<'a, str>,
    pub loc: Option<SourceLocation>,
    // TODO: Not all tokens need these!
    pub no_expand: bool,
    pub treat_as_relax: bool,
}
impl<'a> Token<'a> {
    pub fn new(content: &'a str, loc: impl Into<SourceLocation>) -> Token<'a> {
        Token {
            content: Cow::Borrowed(content),
            loc: Some(loc.into()),
            no_expand: false,
            treat_as_relax: false,
        }
    }

    pub fn new_opt(content: &'a str, loc: Option<impl Into<SourceLocation>>) -> Token<'a> {
        Token {
            content: Cow::Borrowed(content),
            loc: loc.map(Into::into),
            no_expand: false,
            treat_as_relax: false,
        }
    }

    pub fn new_text(content: &'a str) -> Token<'a> {
        Token {
            content: Cow::Borrowed(content),
            loc: None,
            no_expand: false,
            treat_as_relax: false,
        }
    }

    pub fn new_owned(content: String, loc: Option<impl Into<SourceLocation>>) -> Token<'a> {
        Token {
            content: Cow::Owned(content),
            loc: loc.map(Into::into),
            no_expand: false,
            treat_as_relax: false,
        }
    }

    pub fn is_eof(&self) -> bool {
        self.content == "EOF"
    }

    // TODO: We can do better than this.
    pub fn eof(loc: Option<impl Into<SourceLocation>>) -> Token<'static> {
        Token {
            content: Cow::Borrowed("EOF"),
            loc: loc.map(Into::into),
            no_expand: false,
            treat_as_relax: false,
        }
    }

    pub fn into_owned<'b>(self) -> Token<'b> {
        Token {
            content: self.content.into_owned().into(),
            loc: self.loc,
            no_expand: self.no_expand,
            treat_as_relax: self.treat_as_relax,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Token;

    use super::{Lexer, LexerConf};

    #[test]
    fn basic_fraction() {
        let mut lexer = Lexer::new(r#"\frac{a}{b}"#, LexerConf {});

        assert_eq!(lexer.lex().unwrap(), Token::new("\\frac", 0..5));
        assert_eq!(lexer.lex().unwrap(), Token::new("{", 5..6));
        assert_eq!(lexer.lex().unwrap(), Token::new("a", 6..7));
        assert_eq!(lexer.lex().unwrap(), Token::new("}", 7..8));
        assert_eq!(lexer.lex().unwrap(), Token::new("{", 8..9));
        assert_eq!(lexer.lex().unwrap(), Token::new("b", 9..10));
        assert_eq!(lexer.lex().unwrap(), Token::new("}", 10..11));
    }

    #[test]
    fn nontrivial_with_comment() {
        let mut lexer = Lexer::new(
            r#"\left( x \right) \left( x^2 \right) % comment"#,
            LexerConf {},
        );
        assert_eq!(lexer.lex().unwrap(), Token::new("\\left", 0..5));
        assert_eq!(lexer.lex().unwrap(), Token::new("(", 5..6));
        assert_eq!(lexer.lex().unwrap(), Token::new(" ", 6..7));
        assert_eq!(lexer.lex().unwrap(), Token::new("x", 7..8));
        assert_eq!(lexer.lex().unwrap(), Token::new(" ", 8..9));
        assert_eq!(lexer.lex().unwrap(), Token::new("\\right", 9..15));
        assert_eq!(lexer.lex().unwrap(), Token::new(")", 15..16));
        assert_eq!(lexer.lex().unwrap(), Token::new(" ", 16..17));
        assert_eq!(lexer.lex().unwrap(), Token::new("\\left", 17..22));
        assert_eq!(lexer.lex().unwrap(), Token::new("(", 22..23));
        assert_eq!(lexer.lex().unwrap(), Token::new(" ", 23..24));
        assert_eq!(lexer.lex().unwrap(), Token::new("x", 24..25));
        assert_eq!(lexer.lex().unwrap(), Token::new("^", 25..26));
        assert_eq!(lexer.lex().unwrap(), Token::new("2", 26..27));
        assert_eq!(lexer.lex().unwrap(), Token::new(" ", 27..28));
        assert_eq!(lexer.lex().unwrap(), Token::new("\\right", 28..34));
        assert_eq!(lexer.lex().unwrap(), Token::new(")", 34..35));
        assert_eq!(lexer.lex().unwrap(), Token::new(" ", 35..36));
    }
}
