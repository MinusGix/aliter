struct Script {
    name: &'static str,
    blocks: &'static [[u16; 2]],
}

const SCRIPT_DATA: &[Script] = &[
    // Latin characters beyond the Latin-1 characters we have metrics for.
    // Needed for Czech, Hungarian and Turkish text, for example.
    Script {
        name: "latin",
        blocks: &[
            [0x0100, 0x024f], // Latin Extended-A and Latin Extended-B
            [0x0300, 0x036f], // Combining Diacritical marks
        ],
    },
    // The Cyrillic script used by Russian and related languages.
    // A Cyrillic subset used to be supported as explicitly defined
    // symbols in symbols.rs
    Script {
        name: "cyrillic",
        blocks: &[[0x0400, 0x04ff]],
    },
    // Armenian
    Script {
        name: "armenian",
        blocks: &[[0x0530, 0x058F]],
    },
    // The Brahmic scripts of South and Southeast Asia
    // Devanagari (0900–097F)
    // Bengali (0980–09FF)
    // Gurmukhi (0A00–0A7F)
    // Gujarati (0A80–0AFF)
    // Oriya (0B00–0B7F)
    // Tamil (0B80–0BFF)
    // Telugu (0C00–0C7F)
    // Kannada (0C80–0CFF)
    // Malayalam (0D00–0D7F)
    // Sinhala (0D80–0DFF)
    // Thai (0E00–0E7F)
    // Lao (0E80–0EFF)
    // Tibetan (0F00–0FFF)
    // Myanmar (1000–109F)
    Script {
        name: "brahmic",
        blocks: &[[0x0900, 0x109F]],
    },
    Script {
        name: "georgian",
        blocks: &[[0x10A0, 0x10ff]],
    },
    // Chinese and Japanese.
    // The "k" in cjk is for Korean, but we've separated Korean out
    Script {
        name: "cjk",
        blocks: &[
            [0x3000, 0x30FF], // CJK symbols and punctuation, Hiragana, Katakana
            [0x4E00, 0x9FAF], // CJK ideograms
            [0xFF00, 0xFF60], // Fullwidth punctuation
                              // TODO: add halfwidth Katakana and Romanji glyphs
        ],
    },
    // Korean
    Script {
        name: "hangul",
        blocks: &[[0xAC00, 0xD7AF]],
    },
];

// TODO: are these public?
pub(crate) fn script_from_codepoint(codepoint: u16) -> Option<&'static str> {
    SCRIPT_DATA
        .iter()
        .find(|script| {
            script
                .blocks
                .iter()
                .any(|block| codepoint >= block[0] && codepoint <= block[1])
        })
        .map(|script| script.name)
}

pub(crate) fn supported_codepoint(codepoint: u16) -> bool {
    // TODO: katex has a separate allblocks array to make checking for supported codepoint faster
    // should we bother?

    script_from_codepoint(codepoint).is_some()
}
