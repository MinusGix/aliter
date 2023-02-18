use std::collections::HashMap;

use once_cell::sync::Lazy;
use regex::Regex;
use unicode_normalization::UnicodeNormalization;

use crate::expander::Mode;

pub struct Accent {
    pub name: char,
    pub text: &'static str,
    pub math: Option<&'static str>,
}
impl Accent {
    const fn new(name: char, text: &'static str, math: &'static str) -> Accent {
        Accent {
            name,
            text,
            math: Some(math),
        }
    }

    const fn new_text(name: char, text: &'static str) -> Accent {
        Accent {
            name,
            text,
            math: None,
        }
    }

    /// Get the text based on the mode
    pub(crate) fn get_mode(&self, mode: Mode) -> &'static str {
        match mode {
            Mode::Math => self.math.unwrap_or(self.text),
            Mode::Text => self.text,
        }
    }
}

// TODO: are unicode escapes utf8 or utf16?
/// Maps unicode accent characters to their latex equivalent in text and math mode
pub const ACCENTS: &'static [Accent] = &[
    Accent::new('\u{0301}', "\\'", "\\acute"),
    Accent::new('\u{0300}', "\\`", "\\grave"),
    Accent::new('\u{0308}', "\\\"", "\\ddot"),
    Accent::new('\u{0303}', "\\~", "\\tilde"),
    Accent::new('\u{0304}', "\\=", "\\bar"),
    Accent::new('\u{0306}', "\\u", "\\breve"),
    Accent::new('\u{030c}', "\\v", "\\check"),
    Accent::new('\u{0302}', "\\^", "\\hat"),
    Accent::new('\u{0307}', "\\.", "\\dot"),
    Accent::new('\u{030a}', "\\r", "\\mathring"),
    Accent::new_text('\u{030b}', "\\H"),
    Accent::new_text('\u{0327}', "\\c"),
];

pub(crate) fn get_accent(name: char) -> Option<&'static Accent> {
    ACCENTS.iter().find(|accent| accent.name == name)
}

pub static SYMBOLS: Lazy<HashMap<char, String>> = Lazy::new(|| {
    // TODO: Can we do this more efficiently
    let mut res = HashMap::new();
    let letters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZαβγδεϵζηθϑικλμνξοπϖρϱςστυφϕχψωΓΔΘΛΞΠΣΥΦΨΩ";

    for letter in letters.chars() {
        for accent in ACCENTS.iter() {
            // This might be the better method?
            // let combined = unicode_normalization::char::compose(letter, accent.name);

            let combined = format!("{}{}", letter, accent.name);
            let mut normalized = combined.nfc();
            let normalized_first = normalized.next();

            // If there was one resulting character
            if let Some(normalized_first) = normalized_first {
                if normalized.count() == 0 {
                    res.insert(normalized_first, combined.clone());
                }
            }

            for accent2 in ACCENTS.iter() {
                if accent.name == accent2.name {
                    continue;
                }

                let combined2 = format!("{}{}", combined, accent2.name);
                let mut normalized2 = combined2.nfc();
                let normalized2_first = normalized2.next();

                // If there was one resulting character
                if let Some(normalized2_first) = normalized2_first {
                    if normalized2.count() == 0 {
                        res.insert(normalized2_first, combined2);
                    }
                }
            }
        }
    }

    res
});

// TODO: We don't need a regex for this!
pub static SUB_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("^[₊₋₌₍₎₀₁₂₃₄₅₆₇₈₉ₐₑₕᵢⱼₖₗₘₙₒₚᵣₛₜᵤᵥₓᵦᵧᵨᵩᵪ]").unwrap());

/// Maps between (Sub, Sup) characters
pub(crate) const SUBS_AND_SUPS: &'static [(char, char)] = &[
    ('₊', '+'),
    ('₋', '-'),
    ('₌', '='),
    ('₍', '('),
    ('₎', ')'),
    ('₀', '0'),
    ('₁', '1'),
    ('₂', '2'),
    ('₃', '3'),
    ('₄', '4'),
    ('₅', '5'),
    ('₆', '6'),
    ('₇', '7'),
    ('₈', '8'),
    ('₉', '9'),
    ('\u{2090}', 'a'),
    ('\u{2091}', 'e'),
    ('\u{2095}', 'h'),
    ('\u{1D62}', 'i'),
    ('\u{2C7C}', 'j'),
    ('\u{2096}', 'k'),
    ('\u{2097}', 'l'),
    ('\u{2098}', 'm'),
    ('\u{2099}', 'n'),
    ('\u{2092}', 'o'),
    ('\u{209A}', 'p'),
    ('\u{1D63}', 'r'),
    ('\u{209B}', 's'),
    ('\u{209C}', 't'),
    ('\u{1D64}', 'u'),
    ('\u{1D65}', 'v'),
    ('\u{2093}', 'x'),
    ('\u{1D66}', 'β'),
    ('\u{1D67}', 'γ'),
    ('\u{1D68}', 'ρ'),
    ('\u{1D69}', '\u{03d5}'),
    ('\u{1D6A}', 'χ'),
    ('⁺', '+'),
    ('⁻', '-'),
    ('⁼', '='),
    ('⁽', '('),
    ('⁾', ')'),
    ('⁰', '0'),
    ('¹', '1'),
    ('²', '2'),
    ('³', '3'),
    ('⁴', '4'),
    ('⁵', '5'),
    ('⁶', '6'),
    ('⁷', '7'),
    ('⁸', '8'),
    ('⁹', '9'),
    ('\u{1D2C}', 'A'),
    ('\u{1D2E}', 'B'),
    ('\u{1D30}', 'D'),
    ('\u{1D31}', 'E'),
    ('\u{1D33}', 'G'),
    ('\u{1D34}', 'H'),
    ('\u{1D35}', 'I'),
    ('\u{1D36}', 'J'),
    ('\u{1D37}', 'K'),
    ('\u{1D38}', 'L'),
    ('\u{1D39}', 'M'),
    ('\u{1D3A}', 'N'),
    ('\u{1D3C}', 'O'),
    ('\u{1D3E}', 'P'),
    ('\u{1D3F}', 'R'),
    ('\u{1D40}', 'T'),
    ('\u{1D41}', 'U'),
    ('\u{2C7D}', 'V'),
    ('\u{1D42}', 'W'),
    ('\u{1D43}', 'a'),
    ('\u{1D47}', 'b'),
    ('\u{1D9C}', 'c'),
    ('\u{1D48}', 'd'),
    ('\u{1D49}', 'e'),
    ('\u{1DA0}', 'f'),
    ('\u{1D4D}', 'g'),
    ('\u{02B0}', 'h'),
    ('\u{2071}', 'i'),
    ('\u{02B2}', 'j'),
    ('\u{1D4F}', 'k'),
    ('\u{02E1}', 'l'),
    ('\u{1D50}', 'm'),
    ('\u{207F}', 'n'),
    ('\u{1D52}', 'o'),
    ('\u{1D56}', 'p'),
    ('\u{02B3}', 'r'),
    ('\u{02E2}', 's'),
    ('\u{1D57}', 't'),
    ('\u{1D58}', 'u'),
    ('\u{1D5B}', 'v'),
    ('\u{02B7}', 'w'),
    ('\u{02E3}', 'x'),
    ('\u{02B8}', 'y'),
    ('\u{1DBB}', 'z'),
    ('\u{1D5D}', 'β'),
    ('\u{1D5E}', 'γ'),
    ('\u{1D5F}', 'δ'),
    ('\u{1D60}', '\u{03d5}'),
    ('\u{1D61}', 'χ'),
    ('\u{1DBF}', 'θ'),
];

pub(crate) fn find_sub_map_str(sub: &str) -> Option<char> {
    let mut iter = sub.chars();
    let first = iter.next()?;
    // If there's more than one character then it can't be a match
    if iter.next().is_some() {
        return None;
    }
    find_sub_map(first)
}

/// Find the character that a left side maps to
pub(crate) fn find_sub_map(sub: char) -> Option<char> {
    SUBS_AND_SUPS
        .iter()
        .find(|(l, _)| *l == sub)
        .map(|(_, r)| *r)
}
