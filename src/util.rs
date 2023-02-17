use std::{borrow::Cow, fmt::Debug, num::ParseIntError, ops::Range};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    expander::Mode,
    style::{StyleId, DISPLAY_STYLE, SCRIPT_SCRIPT_STYLE, SCRIPT_STYLE, TEXT_STYLE},
    tree::ClassList,
};

#[derive(Clone, PartialEq, Eq)]
pub struct SourceLocation(pub Range<usize>);
impl SourceLocation {
    pub(crate) fn combine(
        start: impl Into<Option<SourceLocation>>,
        end: impl Into<Option<SourceLocation>>,
    ) -> Option<SourceLocation> {
        let start = start.into();
        let end = end.into();
        start
            .zip(end)
            .map(|(start, end)| SourceLocation(start.0.start..end.0.end))
    }
}
impl From<Range<usize>> for SourceLocation {
    fn from(range: Range<usize>) -> Self {
        Self(range)
    }
}
// Custom implementation of debug so that it is a bit less verbose when
// printed
impl Debug for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("SourceLocation({:?})", self.0))
    }
}

/// This mimics JavaScript's `charCodeAt` function.  
/// If the character is a surrogate pair, it will return just the first code point.
pub(crate) fn char_code_for(ch: char) -> u16 {
    let mut buf = [0, 0];
    let ch = ch.encode_utf16(&mut buf);
    ch[0]
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl RGBA {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> RGBA {
        RGBA { r, g, b, a }
    }

    pub const fn into_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}
impl ToString for RGBA {
    fn to_string(&self) -> String {
        format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
    }
}

/// Parse RGB text without a leading #
pub(crate) fn parse_rgb(src: &str) -> Result<[u8; 3], ParseIntError> {
    // TODO: disallow leading +
    debug_assert_eq!(src.len(), 6);
    let pcolor = u32::from_str_radix(src, 16)?;
    let red = (pcolor & 0xFF0000) >> 16;
    let green = (pcolor & 0xFF00) >> 8;
    let blue = pcolor & 0xFF;
    Ok([red as u8, green as u8, blue as u8])
}

/// Parse RGB text of 3 chars without a leading #
pub(crate) fn parse_rgb_3(src: &str) -> Result<[u8; 3], ParseIntError> {
    // TODO: disallow leading +
    debug_assert_eq!(src.len(), 3);
    let pcolor = u16::from_str_radix(src, 16)?;
    let red = (pcolor & 0xF00) >> 8;
    let green = (pcolor & 0xF0) >> 4;
    let blue = pcolor & 0x0F;
    Ok([red as u8, green as u8, blue as u8])
}

/// Parse RGBA text without a leading #
pub(crate) fn parse_rgba(src: &str) -> Result<[u8; 4], ParseIntError> {
    // TODO: disallow leading +
    debug_assert_eq!(src.len(), 6);
    let pcolor = u32::from_str_radix(src, 16)?;
    let red = (pcolor & 0xFF000000) >> 24;
    let green = (pcolor & 0xFF0000) >> 16;
    let blue = (pcolor & 0xFF00) >> 8;
    let alpha = pcolor & 0xFF;
    Ok([red as u8, green as u8, blue as u8, alpha as u8])
}

pub(crate) fn has_class(classes: &ClassList, class: &str) -> bool {
    classes.iter().any(|c| c == class)
}

/// LaTeX display style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Text,
    Display,
    Script,
    ScriptScript,
}
impl Style {
    pub fn into_style_id(self) -> StyleId {
        match self {
            Style::Text => TEXT_STYLE,
            Style::Display => DISPLAY_STYLE,
            Style::Script => SCRIPT_STYLE,
            Style::ScriptScript => SCRIPT_SCRIPT_STYLE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleAuto {
    Style(Style),
    Auto,
}

/// LaTeX argument type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgType {
    /// An HTML color, like "#abc" or "blue"
    Color,
    /// A size-like thing, such as "1em" or "5ex"
    Size,
    /// An url string, in which "\" will be ignored if it precedes
    /// `[#$%&~_^\{}]`
    Url,
    /// A string, allowing single character, percent sign, and nested braces
    Raw,
    /// The same type as the environment that the function being parsed is in  
    /// (e.g. used for the bodies of functions like \textcolor where the first argumet
    /// is special and the second argument is parsed normally)
    Original,
    HBox,
    Primitive,
    /// Node group parsed in a given mode
    Mode(Mode),
}

/// Math font variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FontVariant {
    Bold,
    BoldItalic,
    BoldSansSerif,
    DoubleStruck,
    Fraktur,
    Italic,
    Monospace,
    Normal,
    SansSerif,
    SansSerifBoldItalic,
    SansSerifItalic,
    Script,
}
impl FontVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            FontVariant::Bold => "bold",
            FontVariant::BoldItalic => "bold-italic",
            FontVariant::BoldSansSerif => "bold-sans-serif",
            FontVariant::DoubleStruck => "double-struck",
            FontVariant::Fraktur => "fraktur",
            FontVariant::Italic => "italic",
            FontVariant::Monospace => "monospace",
            FontVariant::Normal => "normal",
            FontVariant::SansSerif => "sans-serif",
            FontVariant::SansSerifBoldItalic => "sans-serif-bold-italic",
            FontVariant::SansSerifItalic => "sans-serif-italic",
            FontVariant::Script => "script",
        }
    }
}

/// Get the first character of the str as a &str, if it can  
/// This is often used because in JS, all characters are strings
/// and so some of our apis accept `&str` but don't accept `char`
pub(crate) fn first_ch_str(text: &str) -> Option<&str> {
    let ch = text.chars().next()?;
    Some(&text[0..ch.len_utf8()])
}

static ESCAPE_REGEX: Lazy<Regex> = Lazy::new(|| {
    const REGEX_TEXT: &str = r#"[&<>"'']"#;

    Regex::new(REGEX_TEXT).unwrap()
});

// hyphenate and escape adapted from KaTeX which adapted them from Facebook's React under Apache 2 license

/// Escapes text to prevent scripting attacks
pub(crate) fn escape(text: &str) -> Cow<'_, str> {
    ESCAPE_REGEX.replace_all(text, |caps: &regex::Captures| -> &'static str {
        // I'm skeptical that this is the best method
        if let Some(first) = caps.iter().next().flatten() {
            match first.as_str() {
                "&" => "&amp;",
                ">" => "&gt;",
                "<" => "&lt;",
                "\"" => "&quot;",
                "'" => "&#x27;",
                _ => "",
            }
        } else {
            // TODO: Warn that we replaced it with nothing
            // I'm assuming that getting rid of the part of text entirely is better than keeping it
            // since this is meant for security filtering
            ""
        }
    })
}

static UPPERCASE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("([A-Z])").unwrap());

pub(crate) fn hyphenate(text: &str) -> String {
    UPPERCASE_REGEX.replace_all(text, "-$1").to_lowercase()
}

/// Find the value associated with a key in a slice of tuples. A poor hashmap.
pub(crate) fn find_assoc_data<K: PartialEq, V>(data: &[(K, V)], key: K) -> Option<&V> {
    data.iter().find(|(k, _)| *k == key).map(|(_, v)| v)
}

#[cfg(test)]
mod tests {
    use crate::util::{char_code_for, hyphenate};

    use super::escape;

    #[test]
    fn test_text_util() {
        // escape
        assert_eq!(escape("abc test"), "abc test");
        assert_eq!(escape("'hello'"), "&#x27;hello&#x27;");
        assert_eq!(escape("test&other"), "test&amp;other");

        // hyphenate
        assert_eq!(hyphenate("testThing"), "test-thing");
        assert_eq!(hyphenate("OTHER"), "-o-t-h-e-r");
    }

    #[test]
    fn test_char_code_for() {
        assert_eq!(char_code_for('a'), 97);
        assert_eq!(char_code_for('A'), 65);
        assert_eq!(char_code_for('0'), 48);
        assert_eq!(char_code_for(' '), 32);
        assert_eq!(char_code_for('😀'), 55357);
        assert_eq!(char_code_for('é'), 233);
        assert_eq!(char_code_for('𝕊'), 55349)
    }
}
