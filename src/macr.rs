use std::{borrow::Cow, collections::HashMap, sync::Arc};

use crate::{
    expander::MacroExpander,
    lexer::{Lexer, LexerConf, Token},
    parser::ParseError,
};

/// The set of backslash macros and letter macros
/// We use `Arc` so that we can be used on multiple threads and because we need to be able to
/// cheaply clone macro replaces so that we can expand them, which needs the structure that contains
/// them.
#[derive(Debug, Clone)]
pub struct Macros<V = Arc<MacroReplace>> {
    // TODO: use Cow<'static, str> since most macro names are known at compile time!
    // We may have to use indexmap for that. Optional crate feature?
    pub(crate) back_macros: HashMap<String, V>,
    // TODO: Logically, most replacements are going to be within typical ascii range, which is very
    // narrow relative to entire unicode range.
    pub(crate) letter_macros: HashMap<char, V>,
}
impl<V> Macros<V> {
    pub fn new_with(back_macros: HashMap<String, V>) -> Self {
        Macros {
            back_macros,
            letter_macros: HashMap::new(),
        }
    }

    pub fn contains_back_macro(&self, name: &str) -> bool {
        self.back_macros.contains_key(name)
    }

    pub fn contains_letter_macro(&self, name: char) -> bool {
        self.letter_macros.contains_key(&name)
    }

    pub fn get_back_macro(&self, name: &str) -> Option<&V> {
        self.back_macros.get(name)
    }

    pub fn get_back_macro_mut(&mut self, name: &str) -> Option<&mut V> {
        self.back_macros.get_mut(name)
    }

    pub fn get_letter_macro(&self, name: char) -> Option<&V> {
        self.letter_macros.get(&name)
    }

    pub fn get_letter_macro_mut(&mut self, name: char) -> Option<&mut V> {
        self.letter_macros.get_mut(&name)
    }

    pub fn take_back_macro(&mut self, name: &str) -> Option<(String, V)> {
        self.back_macros.remove_entry(name)
    }

    pub fn take_letter_macro(&mut self, name: char) -> Option<(char, V)> {
        self.letter_macros.remove_entry(&name)
    }

    pub fn insert_back_macro(&mut self, name: impl Into<String>, repl: V) {
        self.back_macros.insert(name.into(), repl);
    }

    pub fn insert_letter_macro(&mut self, name: char, repl: V) {
        self.letter_macros.insert(name, repl);
    }

    /// Insert all the macros from `other` into this, overwriting any with the same name
    pub fn insert_macros(&mut self, other: Macros<V>) {
        let (back, letter) = other.into_macros_iters();
        self.insert_macros_iter(back, letter)
    }

    pub fn insert_macros_iter(
        &mut self,
        back_macros: impl Iterator<Item = (String, V)>,
        letter_macros: impl Iterator<Item = (char, V)>,
    ) {
        for b in back_macros {
            self.insert_back_macro(b.0, b.1);
        }

        for l in letter_macros {
            self.insert_letter_macro(l.0, l.1);
        }
    }

    pub fn iter_back_macros(&self) -> impl Iterator<Item = (&'_ String, &'_ V)> + '_ {
        self.back_macros.iter()
    }

    pub fn iter_back_macros_mut(&mut self) -> impl Iterator<Item = (&'_ String, &'_ mut V)> + '_ {
        self.back_macros.iter_mut()
    }

    pub fn into_back_macros_iter(self) -> impl Iterator<Item = (String, V)> {
        self.back_macros.into_iter()
    }

    pub fn into_letter_macros_iter(self) -> impl Iterator<Item = (char, V)> {
        self.letter_macros.into_iter()
    }

    pub fn into_macros_iters(
        self,
    ) -> (
        impl Iterator<Item = (String, V)>,
        impl Iterator<Item = (char, V)>,
    ) {
        (self.back_macros.into_iter(), self.letter_macros.into_iter())
    }
}
impl<V> Default for Macros<V> {
    fn default() -> Self {
        Self {
            back_macros: Default::default(),
            letter_macros: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MacroIdentifier<'a> {
    /// "\macro", should include the backslash
    Back(Cow<'a, str>),
    /// Replacing a single literal letter
    Letter(char),
    // TODO: FUnc
}

#[derive(Debug, Clone)]
pub enum MacroVal<'exp, 'text> {
    Text(Cow<'text, str>),
    Expansion(MacroExpansion<'exp>),
}
impl<'exp, 'text> MacroVal<'exp, 'text> {
    pub fn empty_text() -> MacroVal<'exp, 'text> {
        MacroVal::Text(Cow::Borrowed(""))
    }

    pub fn into_expansion(
        self,
        lexer_conf: &LexerConf,
    ) -> Result<MacroExpansion<'exp>, ParseError> {
        match self {
            MacroVal::Text(text) => {
                // TODO: this is more inefficient than it needs to be
                let mut num_args = 0;
                if text.contains('#') {
                    let stripped = text.replace("##", "");
                    while stripped.contains(&format!("#{}", num_args + 1)) {
                        num_args += 1;
                    }
                }

                let mut body_lexer = Lexer::new(&text, lexer_conf.clone());
                let mut tokens = Vec::new();
                loop {
                    let tok = body_lexer.lex()?;
                    if tok.is_eof() {
                        break;
                    }

                    // We unfortunately have to do this to ensure the lifetimes are right
                    tokens.push(tok.into_owned());
                }

                tokens.reverse();

                Ok(MacroExpansion {
                    tokens,
                    num_args,
                    delimiters: None,
                    unexpandable: false,
                })
            }
            MacroVal::Expansion(exp) => Ok(exp),
        }
    }
}
impl<'exp, 'text> From<MacroExpansion<'exp>> for MacroVal<'exp, 'text> {
    fn from(exp: MacroExpansion<'exp>) -> Self {
        MacroVal::Expansion(exp)
    }
}

pub enum MacroReplace {
    /// Replace it with text
    Text(String),
    Expansion(MacroExpansion<'static>),
    Func(
        Box<
            dyn for<'a, 'f> Fn(
                    &mut MacroExpander<'a, 'f>,
                ) -> Result<MacroVal<'a, 'static>, ParseError>
                + Send
                + Sync
                + 'static,
        >,
    ),
}
impl MacroReplace {
    pub fn exec<'a, 'f>(
        &self,
        exp: &mut MacroExpander<'a, 'f>,
    ) -> Result<MacroVal<'a, 'static>, ParseError> {
        match self {
            // TODO: it'd be nice to avoid the clone; Could be done in many cases by allowing macroreplace with a Cow<'static, str>
            MacroReplace::Text(text) => Ok(MacroVal::Text(Cow::Owned(text.clone()))),
            MacroReplace::Expansion(exp) => Ok(MacroVal::Expansion(exp.clone())),
            MacroReplace::Func(f) => (f)(exp),
        }
    }
}
impl std::fmt::Debug for MacroReplace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MacroReplace::Text(text) => f.debug_tuple("Text").field(text).finish(),
            MacroReplace::Expansion(exp) => f.debug_tuple("Expansion").field(exp).finish(),
            MacroReplace::Func(_) => f.debug_tuple("Func").finish(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MacroArg<'a> {
    pub tokens: Vec<Token<'a>>,
    // TODO: This probably can't be EOF?
    pub start: Token<'a>,
    pub end: Token<'a>,
}

// TODO: We could have custom expansions for specific styling macros that are commonly used to
// lessen the amount of text generationg and handling
// Though, since I think they can be redefined (whyyy), we'd either have to
// 1. disallow it, which further breaks compatibility with KaTeX
// 2. Fallback to textual generation in every case where it is used
//    This might not be that bad if it gives the same information, just convert the structure
//    version into expanded tokens/text
#[derive(Debug, Clone)]
pub struct MacroExpansion<'a> {
    // TODO: should this be a smallvec?
    /// Tokens in reverse order
    pub tokens: Vec<Token<'a>>,
    pub num_args: u16,
    pub delimiters: Option<Vec<Vec<Cow<'static, str>>>>,
    pub unexpandable: bool,
}
impl<'a> MacroExpansion<'a> {
    pub fn new(tokens: Vec<Token<'a>>, num_args: u16) -> Self {
        Self {
            tokens,
            num_args,
            delimiters: None,
            unexpandable: false,
        }
    }
}
