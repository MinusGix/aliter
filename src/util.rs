use std::{fmt::Debug, num::ParseIntError, ops::Range};

use crate::expander::Mode;

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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl RGBA {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> RGBA {
        RGBA { r, g, b, a }
    }

    pub fn into_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
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

/// LaTeX display style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Text,
    Display,
    Script,
    ScriptScript,
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

/// Get the first character of the str as a &str, if it can  
/// This is often used because in JS, all characters are strings
/// and so some of our apis accept `&str` but don't accept `char`
pub(crate) fn first_ch_str(text: &str) -> Option<&str> {
    let ch = text.chars().next()?;
    Some(&text[0..ch.len_utf8()])
}
