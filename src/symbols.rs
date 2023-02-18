use std::collections::HashMap;

use once_cell::sync::Lazy;

use crate::{
    expander::Mode,
    parse_node::{
        AccentTokenNode, MathOrdNode, NodeInfo, OpTokenNode, ParseNode, SpacingNode, TextOrdNode,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Font {
    Main,
    Ams,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Atom {
    Bin,
    Close,
    Inner,
    Open,
    Punct,
    Rel,
}
impl Atom {
    pub fn as_str(&self) -> &'static str {
        match self {
            Atom::Bin => "bin",
            Atom::Close => "close",
            Atom::Inner => "inner",
            Atom::Open => "open",
            Atom::Punct => "punct",
            Atom::Rel => "rel",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonAtom {
    AccentToken,
    MathOrd,
    OpToken,
    Spacing,
    TextOrd,
}
impl NonAtom {
    pub(crate) fn make_parse_node(&self, info: NodeInfo, text: String) -> ParseNode {
        match self {
            NonAtom::AccentToken => ParseNode::AccentToken(AccentTokenNode { text, info }),
            NonAtom::MathOrd => ParseNode::MathOrd(MathOrdNode { text, info }),
            NonAtom::OpToken => ParseNode::OpToken(OpTokenNode { text, info }),
            NonAtom::Spacing => ParseNode::Spacing(SpacingNode { text, info }),
            NonAtom::TextOrd => ParseNode::TextOrd(TextOrdNode {
                text: text.into(),
                info,
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Atom(Atom),
    NonAtom(NonAtom),
}

#[derive(Debug, Clone)]
pub struct Symbol<'a> {
    pub font: Font,
    pub group: Group,
    pub replace: Option<&'a str>,
}

pub struct Symbols {
    pub math: HashMap<&'static str, Symbol<'static>>,
    pub text: HashMap<&'static str, Symbol<'static>>,
}
impl Symbols {
    pub fn get(&self, mode: Mode, key: &str) -> Option<&Symbol<'static>> {
        match mode {
            Mode::Math => self.math.get(key),
            Mode::Text => self.text.get(key),
        }
    }

    pub fn contains_key(&self, mode: Mode, key: &str) -> bool {
        match mode {
            Mode::Math => self.math.contains_key(key),
            Mode::Text => self.text.contains_key(key),
        }
    }

    /// Common math
    /// This means `math` and font=`main`
    fn cmath(&mut self, g: Group, r: &'static str, name: &'static str, accept_unicode_char: bool) {
        let symbol = Symbol {
            font: Font::Main,
            group: g,
            replace: Some(r),
        };
        self.math.insert(name, symbol.clone());
        if accept_unicode_char && !r.is_empty() {
            self.math.insert(r, symbol);
        }
    }

    /// Common text
    /// This means `text` and font=`main`
    fn ctext(&mut self, g: Group, r: &'static str, name: &'static str, accept_unicode_char: bool) {
        let symbol = Symbol {
            font: Font::Main,
            group: g,
            replace: Some(r),
        };
        self.text.insert(name, symbol.clone());
        if accept_unicode_char && !r.is_empty() {
            self.text.insert(r, symbol);
        }
    }

    /// Ams math
    /// This means `math` and font=`ams`
    fn amath(&mut self, g: Group, r: &'static str, name: &'static str, accept_unicode_char: bool) {
        let symbol = Symbol {
            font: Font::Ams,
            group: g,
            replace: Some(r),
        };
        self.math.insert(name, symbol.clone());
        if accept_unicode_char && !r.is_empty() {
            self.math.insert(r, symbol);
        }
    }

    /// Ams text
    /// This means `text` and font=`ams`
    fn atext(&mut self, g: Group, r: &'static str, name: &'static str, accept_unicode_char: bool) {
        let symbol = Symbol {
            font: Font::Ams,
            group: g,
            replace: Some(r),
        };
        self.text.insert(name, symbol.clone());
        if accept_unicode_char && !r.is_empty() {
            self.text.insert(r, symbol);
        }
    }
}

pub static SYMBOLS: Lazy<Symbols> = Lazy::new(|| {
    // TODO: with capacity?
    let mut s = Symbols {
        math: HashMap::new(),
        text: HashMap::new(),
    };

    let bin = Group::Atom(Atom::Bin);
    let close = Group::Atom(Atom::Close);
    let inner = Group::Atom(Atom::Inner);
    let open = Group::Atom(Atom::Open);
    let punct = Group::Atom(Atom::Punct);
    let rel = Group::Atom(Atom::Rel);

    let accent = Group::NonAtom(NonAtom::AccentToken);
    let mathord = Group::NonAtom(NonAtom::MathOrd);
    let op = Group::NonAtom(NonAtom::OpToken);
    let spacing = Group::NonAtom(NonAtom::Spacing);
    let textord = Group::NonAtom(NonAtom::TextOrd);

    // Relation Symbols
    s.cmath(rel, "\u{2261}", "\\equiv", true);
    s.cmath(rel, "\u{227a}", "\\prec", true);
    s.cmath(rel, "\u{227b}", "\\succ", true);
    s.cmath(rel, "\u{223c}", "\\sim", true);
    s.cmath(rel, "\u{22a5}", "\\perp", false);
    s.cmath(rel, "\u{2aaf}", "\\preceq", true);
    s.cmath(rel, "\u{2ab0}", "\\succeq", true);
    s.cmath(rel, "\u{2243}", "\\simeq", true);
    s.cmath(rel, "\u{2223}", "\\mid", true);
    s.cmath(rel, "\u{226a}", "\\ll", true);
    s.cmath(rel, "\u{226b}", "\\gg", true);
    s.cmath(rel, "\u{224d}", "\\asymp", true);
    s.cmath(rel, "\u{2225}", "\\parallel", false);
    s.cmath(rel, "\u{22c8}", "\\bowtie", true);
    s.cmath(rel, "\u{2323}", "\\smile", true);
    s.cmath(rel, "\u{2291}", "\\sqsubseteq", true);
    s.cmath(rel, "\u{2292}", "\\sqsupseteq", true);
    s.cmath(rel, "\u{2250}", "\\doteq", true);
    s.cmath(rel, "\u{2322}", "\\frown", true);
    s.cmath(rel, "\u{220b}", "\\ni", true);
    s.cmath(rel, "\u{221d}", "\\propto", true);
    s.cmath(rel, "\u{22a2}", "\\vdash", true);
    s.cmath(rel, "\u{22a3}", "\\dashv", true);
    s.cmath(rel, "\u{220b}", "\\owns", false);

    // Punctuation
    s.cmath(punct, "\u{002e}", "\\ldotp", false);
    s.cmath(punct, "\u{22c5}", "\\cdotp", false);

    // Misc Symbols
    s.cmath(textord, "\u{0023}", "\\#", false);
    s.ctext(textord, "\u{0023}", "\\#", false);
    s.cmath(textord, "\u{0026}", "\\&", false);
    s.ctext(textord, "\u{0026}", "\\&", false);
    s.cmath(textord, "\u{2135}", "\\aleph", true);
    s.cmath(textord, "\u{2200}", "\\forall", true);
    s.cmath(textord, "\u{210f}", "\\hbar", true);
    s.cmath(textord, "\u{2203}", "\\exists", true);
    s.cmath(textord, "\u{2207}", "\\nabla", true);
    s.cmath(textord, "\u{266d}", "\\flat", true);
    s.cmath(textord, "\u{2113}", "\\ell", true);
    s.cmath(textord, "\u{266e}", "\\natural", true);
    s.cmath(textord, "\u{2663}", "\\clubsuit", true);
    s.cmath(textord, "\u{2118}", "\\wp", true);
    s.cmath(textord, "\u{266f}", "\\sharp", true);
    s.cmath(textord, "\u{2662}", "\\diamondsuit", true);
    s.cmath(textord, "\u{211c}", "\\Re", true);
    s.cmath(textord, "\u{2661}", "\\heartsuit", true);
    s.cmath(textord, "\u{2111}", "\\Im", true);
    s.cmath(textord, "\u{2660}", "\\spadesuit", true);
    s.cmath(textord, "\u{00a7}", "\\S", true);
    s.ctext(textord, "\u{00a7}", "\\S", false);
    s.cmath(textord, "\u{00b6}", "\\P", true);
    s.ctext(textord, "\u{00b6}", "\\P", false);

    // Math and Text
    s.cmath(textord, "\u{2020}", "\\dag", false);
    s.ctext(textord, "\u{2020}", "\\dag", false);
    s.ctext(textord, "\u{2020}", "\\textdagger", false);
    s.cmath(textord, "\u{2021}", "\\ddag", false);
    s.ctext(textord, "\u{2021}", "\\ddag", false);
    s.ctext(textord, "\u{2021}", "\\textdaggerdbl", false);

    // Large Delimiters
    s.cmath(close, "\u{23b1}", "\\rmoustache", true);
    s.cmath(open, "\u{23b0}", "\\lmoustache", true);
    s.cmath(close, "\u{27ef}", "\\rgroup", true);
    s.cmath(open, "\u{27ee}", "\\lgroup", true);

    // Binary Operators
    s.cmath(bin, "\u{2213}", "\\mp", true);
    s.cmath(bin, "\u{2296}", "\\ominus", true);
    s.cmath(bin, "\u{228e}", "\\uplus", true);
    s.cmath(bin, "\u{2293}", "\\sqcap", true);
    s.cmath(bin, "\u{2217}", "\\ast", false);
    s.cmath(bin, "\u{2294}", "\\sqcup", true);
    s.cmath(bin, "\u{25ef}", "\\bigcirc", true);
    s.cmath(bin, "\u{2219}", "\\bullet", true);
    s.cmath(bin, "\u{2021}", "\\ddagger", false);
    s.cmath(bin, "\u{2240}", "\\wr", true);
    s.cmath(bin, "\u{2a3f}", "\\amalg", false);
    s.cmath(bin, "\u{0026}", "\\And", false); // from amsmath

    // Arrow Symbols
    s.cmath(rel, "\u{27f5}", "\\longleftarrow", true);
    s.cmath(rel, "\u{21d0}", "\\Leftarrow", true);
    s.cmath(rel, "\u{27f8}", "\\Longleftarrow", true);
    s.cmath(rel, "\u{27f6}", "\\longrightarrow", true);
    s.cmath(rel, "\u{21d2}", "\\Rightarrow", true);
    s.cmath(rel, "\u{27f9}", "\\Longrightarrow", true);
    s.cmath(rel, "\u{2194}", "\\leftrightarrow", true);
    s.cmath(rel, "\u{27f7}", "\\longleftrightarrow", true);
    s.cmath(rel, "\u{21d4}", "\\Leftrightarrow", true);
    s.cmath(rel, "\u{27fa}", "\\Longleftrightarrow", true);
    s.cmath(rel, "\u{21a6}", "\\mapsto", true);
    s.cmath(rel, "\u{27fc}", "\\longmapsto", true);
    s.cmath(rel, "\u{2197}", "\\nearrow", true);
    s.cmath(rel, "\u{21a9}", "\\hookleftarrow", true);
    s.cmath(rel, "\u{21aa}", "\\hookrightarrow", true);
    s.cmath(rel, "\u{2198}", "\\searrow", true);
    s.cmath(rel, "\u{21bc}", "\\leftharpoonup", true);
    s.cmath(rel, "\u{21c0}", "\\rightharpoonup", true);
    s.cmath(rel, "\u{2199}", "\\swarrow", true);
    s.cmath(rel, "\u{21bd}", "\\leftharpoondown", true);
    s.cmath(rel, "\u{21c1}", "\\rightharpoondown", true);
    s.cmath(rel, "\u{2196}", "\\nwarrow", true);
    s.cmath(rel, "\u{21cc}", "\\rightleftharpoons", true);

    // AMS Negated Binary Relations
    s.amath(rel, "\u{226e}", "\\nless", true);
    // Symbol names preceeded by "@" each have a corresponding macro.
    s.amath(rel, "\u{e010}", "\\@nleqslant", false);
    s.amath(rel, "\u{e011}", "\\@nleqq", false);
    s.amath(rel, "\u{2a87}", "\\lneq", true);
    s.amath(rel, "\u{2268}", "\\lneqq", true);
    s.amath(rel, "\u{e00c}", "\\@lvertneqq", false);
    s.amath(rel, "\u{22e6}", "\\lnsim", true);
    s.amath(rel, "\u{2a89}", "\\lnapprox", true);
    s.amath(rel, "\u{2280}", "\\nprec", true);
    // unicode-math maps \u22e0 to \npreccurlyeq. We'll use the AMS synonym.
    s.amath(rel, "\u{22e0}", "\\npreceq", true);
    s.amath(rel, "\u{22e8}", "\\precnsim", true);
    s.amath(rel, "\u{2ab9}", "\\precnapprox", true);
    s.amath(rel, "\u{2241}", "\\nsim", true);
    s.amath(rel, "\u{e006}", "\\@nshortmid", false);
    s.amath(rel, "\u{2224}", "\\nmid", true);
    s.amath(rel, "\u{22ac}", "\\nvdash", true);
    s.amath(rel, "\u{22ad}", "\\nvDash", true);
    s.amath(rel, "\u{22ea}", "\\ntriangleleft", false);
    s.amath(rel, "\u{22ec}", "\\ntrianglelefteq", true);
    s.amath(rel, "\u{228a}", "\\subsetneq", true);
    s.amath(rel, "\u{e01a}", "\\@varsubsetneq", false);
    s.amath(rel, "\u{2acb}", "\\subsetneqq", true);
    s.amath(rel, "\u{e017}", "\\@varsubsetneqq", false);
    s.amath(rel, "\u{226f}", "\\ngtr", true);
    s.amath(rel, "\u{e00f}", "\\@ngeqslant", false);
    s.amath(rel, "\u{e00e}", "\\@ngeqq", false);
    s.amath(rel, "\u{2a88}", "\\gneq", true);
    s.amath(rel, "\u{2269}", "\\gneqq", true);
    s.amath(rel, "\u{e00d}", "\\@gvertneqq", false);
    s.amath(rel, "\u{22e7}", "\\gnsim", true);
    s.amath(rel, "\u{2a8a}", "\\gnapprox", true);
    s.amath(rel, "\u{2281}", "\\nsucc", true);
    // unicode-math maps \u22e1 to \nsucccurlyeq. We'll use the AMS synonym.
    s.amath(rel, "\u{22e1}", "\\nsucceq", true);
    s.amath(rel, "\u{22e9}", "\\succnsim", true);
    s.amath(rel, "\u{2aba}", "\\succnapprox", true);
    // unicode-math maps \u2246 to \simneqq. We'll use the AMS synonym.
    s.amath(rel, "\u{2246}", "\\ncong", true);
    s.amath(rel, "\u{e007}", "\\@nshortparallel", false);
    s.amath(rel, "\u{2226}", "\\nparallel", true);
    s.amath(rel, "\u{22af}", "\\nVDash", true);
    s.amath(rel, "\u{22eb}", "\\ntriangleright", false);
    s.amath(rel, "\u{22ed}", "\\ntrianglerighteq", true);
    s.amath(rel, "\u{e018}", "\\@nsupseteqq", false);
    s.amath(rel, "\u{228b}", "\\supsetneq", true);
    s.amath(rel, "\u{e01b}", "\\@varsupsetneq", false);
    s.amath(rel, "\u{2acc}", "\\supsetneqq", true);
    s.amath(rel, "\u{e019}", "\\@varsupsetneqq", false);
    s.amath(rel, "\u{22ae}", "\\nVdash", true);
    s.amath(rel, "\u{2ab5}", "\\precneqq", true);
    s.amath(rel, "\u{2ab6}", "\\succneqq", true);
    s.amath(rel, "\u{e016}", "\\@nsubseteqq", false);
    s.amath(bin, "\u{22b4}", "\\unlhd", false);
    s.amath(bin, "\u{22b5}", "\\unrhd", false);

    // AMS Negated Arrows
    s.amath(rel, "\u{219a}", "\\nleftarrow", true);
    s.amath(rel, "\u{219b}", "\\nrightarrow", true);
    s.amath(rel, "\u{21cd}", "\\nLeftarrow", true);
    s.amath(rel, "\u{21cf}", "\\nRightarrow", true);
    s.amath(rel, "\u{21ae}", "\\nleftrightarrow", true);
    s.amath(rel, "\u{21ce}", "\\nLeftrightarrow", true);

    // AMS Misc
    s.amath(rel, "\u{25b3}", "\\vartriangle", false);
    s.amath(textord, "\u{210f}", "\\hslash", false);
    s.amath(textord, "\u{25bd}", "\\triangledown", false);
    s.amath(textord, "\u{25ca}", "\\lozenge", false);
    s.amath(textord, "\u{24c8}", "\\circledS", false);
    s.amath(textord, "\u{00ae}", "\\circledR", false);
    s.atext(textord, "\u{00ae}", "\\circledR", false);
    s.amath(textord, "\u{2221}", "\\measuredangle", true);
    s.amath(textord, "\u{2204}", "\\nexists", false);
    s.amath(textord, "\u{2127}", "\\mho", false);
    s.amath(textord, "\u{2132}", "\\Finv", true);
    s.amath(textord, "\u{2141}", "\\Game", true);
    s.amath(textord, "\u{2035}", "\\backprime", false);
    s.amath(textord, "\u{25b2}", "\\blacktriangle", false);
    s.amath(textord, "\u{25bc}", "\\blacktriangledown", false);
    s.amath(textord, "\u{25a0}", "\\blacksquare", false);
    s.amath(textord, "\u{29eb}", "\\blacklozenge", false);
    s.amath(textord, "\u{2605}", "\\bigstar", false);
    s.amath(textord, "\u{2222}", "\\sphericalangle", true);
    s.amath(textord, "\u{2201}", "\\complement", true);
    // unicode-math maps U+F0 to \matheth. We map to AMS function \eth
    s.amath(textord, "\u{00f0}", "\\eth", true);
    s.ctext(textord, "\u{00f0}", "\u{00f0}", false);
    s.amath(textord, "\u{2571}", "\\diagup", false);
    s.amath(textord, "\u{2572}", "\\diagdown", false);
    s.amath(textord, "\u{25a1}", "\\square", false);
    s.amath(textord, "\u{25a1}", "\\Box", false);
    s.amath(textord, "\u{25ca}", "\\Diamond", false);
    // unicode-math maps U+A5 to \mathyen. We map to AMS function \yen
    s.amath(textord, "\u{00a5}", "\\yen", true);
    s.atext(textord, "\u{00a5}", "\\yen", true);
    s.amath(textord, "\u{2713}", "\\checkmark", true);
    s.atext(textord, "\u{2713}", "\\checkmark", false);

    // AMS Hebrew
    s.amath(textord, "\u{2136}", "\\beth", true);
    s.amath(textord, "\u{2138}", "\\daleth", true);
    s.amath(textord, "\u{2137}", "\\gimel", true);

    // AMS Greek
    s.amath(textord, "\u{03dd}", "\\digamma", true);
    s.amath(textord, "\u{03f0}", "\\varkappa", false);

    // AMS Delimiters
    s.amath(open, "\u{250c}", "\\@ulcorner", true);
    s.amath(close, "\u{2510}", "\\@urcorner", true);
    s.amath(open, "\u{2514}", "\\@llcorner", true);
    s.amath(close, "\u{2518}", "\\@lrcorner", true);

    // AMS Binary Relations
    s.amath(rel, "\u{2266}", "\\leqq", true);
    s.amath(rel, "\u{2a7d}", "\\leqslant", true);
    s.amath(rel, "\u{2a95}", "\\eqslantless", true);
    s.amath(rel, "\u{2272}", "\\lesssim", true);
    s.amath(rel, "\u{2a85}", "\\lessapprox", true);
    s.amath(rel, "\u{224a}", "\\approxeq", true);
    s.amath(bin, "\u{22d6}", "\\lessdot", false);
    s.amath(rel, "\u{22d8}", "\\lll", true);
    s.amath(rel, "\u{2276}", "\\lessgtr", true);
    s.amath(rel, "\u{22da}", "\\lesseqgtr", true);
    s.amath(rel, "\u{2a8b}", "\\lesseqqgtr", true);
    s.amath(rel, "\u{2251}", "\\doteqdot", false);
    s.amath(rel, "\u{2253}", "\\risingdotseq", true);
    s.amath(rel, "\u{2252}", "\\fallingdotseq", true);
    s.amath(rel, "\u{223d}", "\\backsim", true);
    s.amath(rel, "\u{22cd}", "\\backsimeq", true);
    s.amath(rel, "\u{2ac5}", "\\subseteqq", true);
    s.amath(rel, "\u{22d0}", "\\Subset", true);
    s.amath(rel, "\u{228f}", "\\sqsubset", true);
    s.amath(rel, "\u{227c}", "\\preccurlyeq", true);
    s.amath(rel, "\u{22de}", "\\curlyeqprec", true);
    s.amath(rel, "\u{227e}", "\\precsim", true);
    s.amath(rel, "\u{2ab7}", "\\precapprox", true);
    s.amath(rel, "\u{22b2}", "\\vartriangleleft", false);
    s.amath(rel, "\u{22b4}", "\\trianglelefteq", false);
    s.amath(rel, "\u{22a8}", "\\vDash", true);
    s.amath(rel, "\u{22aa}", "\\Vvdash", true);
    s.amath(rel, "\u{2323}", "\\smallsmile", false);
    s.amath(rel, "\u{2322}", "\\smallfrown", false);
    s.amath(rel, "\u{224f}", "\\bumpeq", true);
    s.amath(rel, "\u{224e}", "\\Bumpeq", true);
    s.amath(rel, "\u{2267}", "\\geqq", true);
    s.amath(rel, "\u{2a7e}", "\\geqslant", true);
    s.amath(rel, "\u{2a96}", "\\eqslantgtr", true);
    s.amath(rel, "\u{2273}", "\\gtrsim", true);
    s.amath(rel, "\u{2a86}", "\\gtrapprox", true);
    s.amath(bin, "\u{22d7}", "\\gtrdot", false);
    s.amath(rel, "\u{22d9}", "\\ggg", true);
    s.amath(rel, "\u{2277}", "\\gtrless", true);
    s.amath(rel, "\u{22db}", "\\gtreqless", true);
    s.amath(rel, "\u{2a8c}", "\\gtreqqless", true);
    s.amath(rel, "\u{2256}", "\\eqcirc", true);
    s.amath(rel, "\u{2257}", "\\circeq", true);
    s.amath(rel, "\u{225c}", "\\triangleq", true);
    s.amath(rel, "\u{223c}", "\\thicksim", false);
    s.amath(rel, "\u{2248}", "\\thickapprox", false);
    s.amath(rel, "\u{2ac6}", "\\supseteqq", true);
    s.amath(rel, "\u{22d1}", "\\Supset", true);
    s.amath(rel, "\u{2290}", "\\sqsupset", true);
    s.amath(rel, "\u{227d}", "\\succcurlyeq", true);
    s.amath(rel, "\u{22df}", "\\curlyeqsucc", true);
    s.amath(rel, "\u{227f}", "\\succsim", true);
    s.amath(rel, "\u{2ab8}", "\\succapprox", true);
    s.amath(rel, "\u{22b3}", "\\vartriangleright", false);
    s.amath(rel, "\u{22b5}", "\\trianglerighteq", false);
    s.amath(rel, "\u{22a9}", "\\Vdash", true);
    s.amath(rel, "\u{2223}", "\\shortmid", false);
    s.amath(rel, "\u{2225}", "\\shortparallel", false);
    s.amath(rel, "\u{226c}", "\\between", true);
    s.amath(rel, "\u{22d4}", "\\pitchfork", true);
    s.amath(rel, "\u{221d}", "\\varpropto", false);
    s.amath(rel, "\u{25c0}", "\\blacktriangleleft", false);
    // unicode-math says that \therefore is a mathord atom.
    // We kept the amssymb atom type, which is rel.
    s.amath(rel, "\u{2234}", "\\therefore", true);
    s.amath(rel, "\u{220d}", "\\backepsilon", false);
    s.amath(rel, "\u{25b6}", "\\blacktriangleright", false);
    // unicode-math says that \because is a mathord atom.
    // We kept the amssymb atom type, which is rel.
    s.amath(rel, "\u{2235}", "\\because", true);
    s.amath(rel, "\u{22d8}", "\\llless", false);
    s.amath(rel, "\u{22d9}", "\\gggtr", false);
    s.amath(bin, "\u{22b2}", "\\lhd", false);
    s.amath(bin, "\u{22b3}", "\\rhd", false);
    s.amath(rel, "\u{2242}", "\\eqsim", true);
    s.cmath(rel, "\u{22c8}", "\\Join", false);
    s.amath(rel, "\u{2251}", "\\Doteq", true);

    // AMS Binary Operators
    s.amath(bin, "\u{2214}", "\\dotplus", true);
    s.amath(bin, "\u{2216}", "\\smallsetminus", false);
    s.amath(bin, "\u{22d2}", "\\Cap", true);
    s.amath(bin, "\u{22d3}", "\\Cup", true);
    s.amath(bin, "\u{2a5e}", "\\doublebarwedge", true);
    s.amath(bin, "\u{229f}", "\\boxminus", true);
    s.amath(bin, "\u{229e}", "\\boxplus", true);
    s.amath(bin, "\u{22c7}", "\\divideontimes", true);
    s.amath(bin, "\u{22c9}", "\\ltimes", true);
    s.amath(bin, "\u{22ca}", "\\rtimes", true);
    s.amath(bin, "\u{22cb}", "\\leftthreetimes", true);
    s.amath(bin, "\u{22cc}", "\\rightthreetimes", true);
    s.amath(bin, "\u{22cf}", "\\curlywedge", true);
    s.amath(bin, "\u{22ce}", "\\curlyvee", true);
    s.amath(bin, "\u{229d}", "\\circleddash", true);
    s.amath(bin, "\u{229b}", "\\circledast", true);
    s.amath(bin, "\u{22c5}", "\\centerdot", false);
    s.amath(bin, "\u{22ba}", "\\intercal", true);
    s.amath(bin, "\u{22d2}", "\\doublecap", false);
    s.amath(bin, "\u{22d3}", "\\doublecup", false);
    s.amath(bin, "\u{22a0}", "\\boxtimes", true);

    // AMS Arrows
    // Note: unicode-math maps \u21e2 to their own function \rightdasharrow.
    // We'll map it to AMS function \dashrightarrow. It produces the same atom.
    s.amath(rel, "\u{21e2}", "\\dashrightarrow", true);
    // unicode-math maps \u21e0 to \leftdasharrow. We'll use the AMS synonym.
    s.amath(rel, "\u{21e0}", "\\dashleftarrow", true);
    s.amath(rel, "\u{21c7}", "\\leftleftarrows", true);
    s.amath(rel, "\u{21c6}", "\\leftrightarrows", true);
    s.amath(rel, "\u{21da}", "\\Lleftarrow", true);
    s.amath(rel, "\u{219e}", "\\twoheadleftarrow", true);
    s.amath(rel, "\u{21a2}", "\\leftarrowtail", true);
    s.amath(rel, "\u{21ab}", "\\looparrowleft", true);
    s.amath(rel, "\u{21cb}", "\\leftrightharpoons", true);
    s.amath(rel, "\u{21b6}", "\\curvearrowleft", true);
    // unicode-math maps \u21ba to \acwopencirclearrow. We'll use the AMS synonym.
    s.amath(rel, "\u{21ba}", "\\circlearrowleft", true);
    s.amath(rel, "\u{21b0}", "\\Lsh", true);
    s.amath(rel, "\u{21c8}", "\\upuparrows", true);
    s.amath(rel, "\u{21bf}", "\\upharpoonleft", true);
    s.amath(rel, "\u{21c3}", "\\downharpoonleft", true);
    s.cmath(rel, "\u{22b6}", "\\origof", true); // not in font
    s.cmath(rel, "\u{22b7}", "\\imageof", true); // not in font
    s.amath(rel, "\u{22b8}", "\\multimap", true);
    s.amath(rel, "\u{21ad}", "\\leftrightsquigarrow", true);
    s.amath(rel, "\u{21c9}", "\\rightrightarrows", true);
    s.amath(rel, "\u{21c4}", "\\rightleftarrows", true);
    s.amath(rel, "\u{21a0}", "\\twoheadrightarrow", true);
    s.amath(rel, "\u{21a3}", "\\rightarrowtail", true);
    s.amath(rel, "\u{21ac}", "\\looparrowright", true);
    s.amath(rel, "\u{21b7}", "\\curvearrowright", true);
    // unicode-math maps \u21bb to \cwopencirclearrow. We'll use the AMS synonym.
    s.amath(rel, "\u{21bb}", "\\circlearrowright", true);
    s.amath(rel, "\u{21b1}", "\\Rsh", true);
    s.amath(rel, "\u{21ca}", "\\downdownarrows", true);
    s.amath(rel, "\u{21be}", "\\upharpoonright", true);
    s.amath(rel, "\u{21c2}", "\\downharpoonright", true);
    s.amath(rel, "\u{21dd}", "\\rightsquigarrow", true);
    s.amath(rel, "\u{21dd}", "\\leadsto", false);
    s.amath(rel, "\u{21db}", "\\Rrightarrow", true);
    s.amath(rel, "\u{21be}", "\\restriction", false);

    s.cmath(textord, "\u{2018}", "`", false);
    s.cmath(textord, "$", "\\$", false);
    s.ctext(textord, "$", "\\$", false);
    s.ctext(textord, "$", "\\textdollar", false);
    s.cmath(textord, "%", "\\%", false);
    s.ctext(textord, "%", "\\%", false);
    s.cmath(textord, "_", "\\_", false);
    s.ctext(textord, "_", "\\_", false);
    s.ctext(textord, "_", "\\textunderscore", false);
    s.cmath(textord, "\u{2220}", "\\angle", true);
    s.cmath(textord, "\u{221e}", "\\infty", true);
    s.cmath(textord, "\u{2032}", "\\prime", false);
    s.cmath(textord, "\u{25b3}", "\\triangle", false);
    s.cmath(textord, "\u{0393}", "\\Gamma", true);
    s.cmath(textord, "\u{0394}", "\\Delta", true);
    s.cmath(textord, "\u{0398}", "\\Theta", true);
    s.cmath(textord, "\u{039b}", "\\Lambda", true);
    s.cmath(textord, "\u{039e}", "\\Xi", true);
    s.cmath(textord, "\u{03a0}", "\\Pi", true);
    s.cmath(textord, "\u{03a3}", "\\Sigma", true);
    s.cmath(textord, "\u{03a5}", "\\Upsilon", true);
    s.cmath(textord, "\u{03a6}", "\\Phi", true);
    s.cmath(textord, "\u{03a8}", "\\Psi", true);
    s.cmath(textord, "\u{03a9}", "\\Omega", true);
    s.cmath(textord, "A", "\u{0391}", false);
    s.cmath(textord, "B", "\u{0392}", false);
    s.cmath(textord, "E", "\u{0395}", false);
    s.cmath(textord, "Z", "\u{0396}", false);
    s.cmath(textord, "H", "\u{0397}", false);
    s.cmath(textord, "I", "\u{0399}", false);
    s.cmath(textord, "K", "\u{039A}", false);
    s.cmath(textord, "M", "\u{039C}", false);
    s.cmath(textord, "N", "\u{039D}", false);
    s.cmath(textord, "O", "\u{039F}", false);
    s.cmath(textord, "P", "\u{03A1}", false);
    s.cmath(textord, "T", "\u{03A4}", false);
    s.cmath(textord, "X", "\u{03A7}", false);
    s.cmath(textord, "\u{00ac}", "\\neg", true);
    s.cmath(textord, "\u{00ac}", "\\lnot", false);
    s.cmath(textord, "\u{22a4}", "\\top", false);
    s.cmath(textord, "\u{22a5}", "\\bot", false);
    s.cmath(textord, "\u{2205}", "\\emptyset", false);
    s.amath(textord, "\u{2205}", "\\varnothing", false);
    s.cmath(mathord, "\u{03b1}", "\\alpha", true);
    s.cmath(mathord, "\u{03b2}", "\\beta", true);
    s.cmath(mathord, "\u{03b3}", "\\gamma", true);
    s.cmath(mathord, "\u{03b4}", "\\delta", true);
    s.cmath(mathord, "\u{03f5}", "\\epsilon", true);
    s.cmath(mathord, "\u{03b6}", "\\zeta", true);
    s.cmath(mathord, "\u{03b7}", "\\eta", true);
    s.cmath(mathord, "\u{03b8}", "\\theta", true);
    s.cmath(mathord, "\u{03b9}", "\\iota", true);
    s.cmath(mathord, "\u{03ba}", "\\kappa", true);
    s.cmath(mathord, "\u{03bb}", "\\lambda", true);
    s.cmath(mathord, "\u{03bc}", "\\mu", true);
    s.cmath(mathord, "\u{03bd}", "\\nu", true);
    s.cmath(mathord, "\u{03be}", "\\xi", true);
    s.cmath(mathord, "\u{03bf}", "\\omicron", true);
    s.cmath(mathord, "\u{03c0}", "\\pi", true);
    s.cmath(mathord, "\u{03c1}", "\\rho", true);
    s.cmath(mathord, "\u{03c3}", "\\sigma", true);
    s.cmath(mathord, "\u{03c4}", "\\tau", true);
    s.cmath(mathord, "\u{03c5}", "\\upsilon", true);
    s.cmath(mathord, "\u{03d5}", "\\phi", true);
    s.cmath(mathord, "\u{03c7}", "\\chi", true);
    s.cmath(mathord, "\u{03c8}", "\\psi", true);
    s.cmath(mathord, "\u{03c9}", "\\omega", true);
    s.cmath(mathord, "\u{03b5}", "\\varepsilon", true);
    s.cmath(mathord, "\u{03d1}", "\\vartheta", true);
    s.cmath(mathord, "\u{03d6}", "\\varpi", true);
    s.cmath(mathord, "\u{03f1}", "\\varrho", true);
    s.cmath(mathord, "\u{03c2}", "\\varsigma", true);
    s.cmath(mathord, "\u{03c6}", "\\varphi", true);
    s.cmath(bin, "\u{2217}", "*", true);
    s.cmath(bin, "+", "+", false);
    s.cmath(bin, "\u{2212}", "-", true);
    s.cmath(bin, "\u{22c5}", "\\cdot", true);
    s.cmath(bin, "\u{2218}", "\\circ", true);
    s.cmath(bin, "\u{00f7}", "\\div", true);
    s.cmath(bin, "\u{00b1}", "\\pm", true);
    s.cmath(bin, "\u{00d7}", "\\times", true);
    s.cmath(bin, "\u{2229}", "\\cap", true);
    s.cmath(bin, "\u{222a}", "\\cup", true);
    s.cmath(bin, "\u{2216}", "\\setminus", true);
    s.cmath(bin, "\u{2227}", "\\land", false);
    s.cmath(bin, "\u{2228}", "\\lor", false);
    s.cmath(bin, "\u{2227}", "\\wedge", true);
    s.cmath(bin, "\u{2228}", "\\vee", true);
    s.cmath(textord, "\u{221a}", "\\surd", false);
    s.cmath(open, "\u{27e8}", "\\langle", true);
    s.cmath(open, "\u{2223}", "\\lvert", false);
    s.cmath(open, "\u{2225}", "\\lVert", false);
    s.cmath(close, "?", "?", false);
    s.cmath(close, "!", "!", false);
    s.cmath(close, "\u{27e9}", "\\rangle", true);
    s.cmath(close, "\u{2223}", "\\rvert", false);
    s.cmath(close, "\u{2225}", "\\rVert", false);
    s.cmath(rel, "=", "=", false);
    s.cmath(rel, ":", ":", false);
    s.cmath(rel, "\u{2248}", "\\approx", true);
    s.cmath(rel, "\u{2245}", "\\cong", true);
    s.cmath(rel, "\u{2265}", "\\ge", false);
    s.cmath(rel, "\u{2265}", "\\geq", true);
    s.cmath(rel, "\u{2190}", "\\gets", false);
    s.cmath(rel, ">", "\\gt", true);
    s.cmath(rel, "\u{2208}", "\\in", true);
    s.cmath(rel, "\u{e020}", "\\@not", false);
    s.cmath(rel, "\u{2282}", "\\subset", true);
    s.cmath(rel, "\u{2283}", "\\supset", true);
    s.cmath(rel, "\u{2286}", "\\subseteq", true);
    s.cmath(rel, "\u{2287}", "\\supseteq", true);
    s.amath(rel, "\u{2288}", "\\nsubseteq", true);
    s.amath(rel, "\u{2289}", "\\nsupseteq", true);
    s.cmath(rel, "\u{22a8}", "\\models", false);
    s.cmath(rel, "\u{2190}", "\\leftarrow", true);
    s.cmath(rel, "\u{2264}", "\\le", false);
    s.cmath(rel, "\u{2264}", "\\leq", true);
    s.cmath(rel, "<", "\\lt", true);
    s.cmath(rel, "\u{2192}", "\\rightarrow", true);
    s.cmath(rel, "\u{2192}", "\\to", false);
    s.amath(rel, "\u{2271}", "\\ngeq", true);
    s.amath(rel, "\u{2270}", "\\nleq", true);
    s.cmath(spacing, "\u{00a0}", "\\ ", false);
    s.cmath(spacing, "\u{00a0}", "\\space", false);
    // Ref: LaTeX Source 2e: \DeclareRobustCommand{\nobreakspace}{%
    s.cmath(spacing, "\u{00a0}", "\\nobreakspace", false);
    s.ctext(spacing, "\u{00a0}", "\\ ", false);
    s.ctext(spacing, "\u{00a0}", " ", false);
    s.ctext(spacing, "\u{00a0}", "\\space", false);
    s.ctext(spacing, "\u{00a0}", "\\nobreakspace", false);
    // TODO: Is it correct to translate null to 0x00?
    s.cmath(spacing, "\u{0000}", "\\nobreak", false);
    s.cmath(spacing, "\u{0000}", "\\allowbreak", false);
    s.cmath(punct, ",", ",", false);
    s.cmath(punct, ";", ";", false);
    s.amath(bin, "\u{22bc}", "\\barwedge", true);
    s.amath(bin, "\u{22bb}", "\\veebar", true);
    s.cmath(bin, "\u{2299}", "\\odot", true);
    s.cmath(bin, "\u{2295}", "\\oplus", true);
    s.cmath(bin, "\u{2297}", "\\otimes", true);
    s.cmath(textord, "\u{2202}", "\\partial", true);
    s.cmath(bin, "\u{2298}", "\\oslash", true);
    s.amath(bin, "\u{229a}", "\\circledcirc", true);
    s.amath(bin, "\u{22a1}", "\\boxdot", true);
    s.cmath(bin, "\u{25b3}", "\\bigtriangleup", false);
    s.cmath(bin, "\u{25bd}", "\\bigtriangledown", false);
    s.cmath(bin, "\u{2020}", "\\dagger", false);
    s.cmath(bin, "\u{22c4}", "\\diamond", false);
    s.cmath(bin, "\u{22c6}", "\\star", false);
    s.cmath(bin, "\u{25c3}", "\\triangleleft", false);
    s.cmath(bin, "\u{25b9}", "\\triangleright", false);
    s.cmath(open, "{", "\\{", false);
    s.ctext(textord, "{", "\\{", false);
    s.ctext(textord, "{", "\\textbraceleft", false);
    s.cmath(close, "}", "\\}", false);
    s.ctext(textord, "}", "\\}", false);
    s.ctext(textord, "}", "\\textbraceright", false);
    s.cmath(open, "{", "\\lbrace", false);
    s.cmath(close, "}", "\\rbrace", false);
    s.cmath(open, "[", "\\lbrack", true);
    s.ctext(textord, "[", "\\lbrack", true);
    s.cmath(close, "]", "\\rbrack", true);
    s.ctext(textord, "]", "\\rbrack", true);
    s.cmath(open, "(", "\\lparen", true);
    s.cmath(close, ")", "\\rparen", true);
    s.ctext(textord, "<", "\\textless", true); // in T1 fontenc
    s.ctext(textord, ">", "\\textgreater", true); // in T1 fontenc
    s.cmath(open, "\u{230a}", "\\lfloor", true);
    s.cmath(close, "\u{230b}", "\\rfloor", true);
    s.cmath(open, "\u{2308}", "\\lceil", true);
    s.cmath(close, "\u{2309}", "\\rceil", true);
    s.cmath(textord, "\\", "\\backslash", false);
    s.cmath(textord, "\u{2223}", "|", false);
    s.cmath(textord, "\u{2223}", "\\vert", false);
    s.ctext(textord, "|", "\\textbar", true); // in T1 fontenc
    s.cmath(textord, "\u{2225}", "\\|", false);
    s.cmath(textord, "\u{2225}", "\\Vert", false);
    s.ctext(textord, "\u{2225}", "\\textbardbl", false);
    s.ctext(textord, "~", "\\textasciitilde", false);
    s.ctext(textord, "\\", "\\textbackslash", false);
    s.ctext(textord, "^", "\\textasciicircum", false);
    s.cmath(rel, "\u{2191}", "\\uparrow", true);
    s.cmath(rel, "\u{21d1}", "\\Uparrow", true);
    s.cmath(rel, "\u{2193}", "\\downarrow", true);
    s.cmath(rel, "\u{21d3}", "\\Downarrow", true);
    s.cmath(rel, "\u{2195}", "\\updownarrow", true);
    s.cmath(rel, "\u{21d5}", "\\Updownarrow", true);
    s.cmath(op, "\u{2210}", "\\coprod", false);
    s.cmath(op, "\u{22c1}", "\\bigvee", false);
    s.cmath(op, "\u{22c0}", "\\bigwedge", false);
    s.cmath(op, "\u{2a04}", "\\biguplus", false);
    s.cmath(op, "\u{22c2}", "\\bigcap", false);
    s.cmath(op, "\u{22c3}", "\\bigcup", false);
    s.cmath(op, "\u{222b}", "\\int", false);
    s.cmath(op, "\u{222b}", "\\intop", false);
    s.cmath(op, "\u{222c}", "\\iint", false);
    s.cmath(op, "\u{222d}", "\\iiint", false);
    s.cmath(op, "\u{220f}", "\\prod", false);
    s.cmath(op, "\u{2211}", "\\sum", false);
    s.cmath(op, "\u{2a02}", "\\bigotimes", false);
    s.cmath(op, "\u{2a01}", "\\bigoplus", false);
    s.cmath(op, "\u{2a00}", "\\bigodot", false);
    s.cmath(op, "\u{222e}", "\\oint", false);
    s.cmath(op, "\u{222f}", "\\oiint", false);
    s.cmath(op, "\u{2230}", "\\oiiint", false);
    s.cmath(op, "\u{2a06}", "\\bigsqcup", false);
    s.cmath(op, "\u{222b}", "\\smallint", false);
    s.ctext(inner, "\u{2026}", "\\textellipsis", false);
    s.cmath(inner, "\u{2026}", "\\mathellipsis", false);
    s.ctext(inner, "\u{2026}", "\\ldots", true);
    s.cmath(inner, "\u{2026}", "\\ldots", true);
    s.cmath(inner, "\u{22ef}", "\\@cdots", true);
    s.cmath(inner, "\u{22f1}", "\\ddots", true);
    s.cmath(textord, "\u{22ee}", "\\varvdots", false); // \vdots is a macro
    s.cmath(accent, "\u{02ca}", "\\acute", false);
    s.cmath(accent, "\u{02cb}", "\\grave", false);
    s.cmath(accent, "\u{00a8}", "\\ddot", false);
    s.cmath(accent, "\u{007e}", "\\tilde", false);
    s.cmath(accent, "\u{02c9}", "\\bar", false);
    s.cmath(accent, "\u{02d8}", "\\breve", false);
    s.cmath(accent, "\u{02c7}", "\\check", false);
    s.cmath(accent, "\u{005e}", "\\hat", false);
    s.cmath(accent, "\u{20d7}", "\\vec", false);
    s.cmath(accent, "\u{02d9}", "\\dot", false);
    s.cmath(accent, "\u{02da}", "\\mathring", false);
    // \imath and \jmath should be invariant to \mathrm, \mathbf, etc., so use PUA
    s.cmath(mathord, "\u{e131}", "\\@imath", false);
    s.cmath(mathord, "\u{e237}", "\\@jmath", false);
    s.cmath(textord, "\u{0131}", "\u{0131}", false);
    s.cmath(textord, "\u{0237}", "\u{0237}", false);
    s.ctext(textord, "\u{0131}", "\\i", true);
    s.ctext(textord, "\u{0237}", "\\j", true);
    s.ctext(textord, "\u{00df}", "\\ss", true);
    s.ctext(textord, "\u{00e6}", "\\ae", true);
    s.ctext(textord, "\u{0153}", "\\oe", true);
    s.ctext(textord, "\u{00f8}", "\\o", true);
    s.ctext(textord, "\u{00c6}", "\\AE", true);
    s.ctext(textord, "\u{0152}", "\\OE", true);
    s.ctext(textord, "\u{00d8}", "\\O", true);
    s.ctext(accent, "\u{02ca}", "\\'", false); // acute
    s.ctext(accent, "\u{02cb}", "\\`", false); // grave
    s.ctext(accent, "\u{02c6}", "\\^", false); // circumflex
    s.ctext(accent, "\u{02dc}", "\\~", false); // tilde
    s.ctext(accent, "\u{02c9}", "\\=", false); // macron
    s.ctext(accent, "\u{02d8}", "\\u", false); // breve
    s.ctext(accent, "\u{02d9}", "\\.", false); // dot above
    s.ctext(accent, "\u{00b8}", "\\c", false); // cedilla
    s.ctext(accent, "\u{02da}", "\\r", false); // ring above
    s.ctext(accent, "\u{02c7}", "\\v", false); // caron
    s.ctext(accent, "\u{00a8}", "\\\"", false); // diaresis
    s.ctext(accent, "\u{02dd}", "\\H", false); // double acute
    s.ctext(accent, "\u{25ef}", "\\textcircled", false); // \bigcirc glyph

    s.ctext(textord, "\u{2013}", "--", true);
    s.ctext(textord, "\u{2013}", "\\textendash", false);
    s.ctext(textord, "\u{2014}", "---", true);
    s.ctext(textord, "\u{2014}", "\\textemdash", false);
    s.ctext(textord, "\u{2018}", "`", true);
    s.ctext(textord, "\u{2018}", "\\textquoteleft", false);
    s.ctext(textord, "\u{2019}", "'", true);
    s.ctext(textord, "\u{2019}", "\\textquoteright", false);
    s.ctext(textord, "\u{201c}", "``", true);
    s.ctext(textord, "\u{201c}", "\\textquotedblleft", false);
    s.ctext(textord, "\u{201d}", "''", true);
    s.ctext(textord, "\u{201d}", "\\textquotedblright", false);
    //  \degree from gensymb package
    s.cmath(textord, "\u{00b0}", "\\degree", true);
    s.ctext(textord, "\u{00b0}", "\\degree", false);
    // \textdegree from inputenc package
    s.ctext(textord, "\u{00b0}", "\\textdegree", true);
    // TODO: In LaTeX, \pounds can generate a different character in text and math
    // mode, but among our fonts, only Main-Regular defines this character "163".
    s.cmath(textord, "\u{00a3}", "\\pounds", false);
    s.cmath(textord, "\u{00a3}", "\\mathsterling", true);
    s.ctext(textord, "\u{00a3}", "\\pounds", false);
    s.ctext(textord, "\u{00a3}", "\\textsterling", true);
    s.cmath(textord, "\u{2720}", "\\maltese", false);
    s.ctext(textord, "\u{2720}", "\\maltese", false);

    // TODO: rather than this kindof absurd individual additions, we can just have checks like
    // 'is digit', 'is alphabetic' for most of them, to avoid storing and processing as much
    let math_text_symbols = [
        "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "/", "@", ".", "\"",
    ];
    for sym in math_text_symbols {
        s.cmath(textord, sym, sym, false);
    }

    let text_symbols = [
        "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "!", "@", "*", "(", ")", "-", "=", "+",
        "\"", ";", ":", "?", "/", ".", ",",
    ];
    for sym in text_symbols {
        s.ctext(textord, sym, sym, false);
    }

    let letters = [
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R",
        "S", "T", "U", "V", "W", "X", "Y", "Z", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j",
        "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
    ];
    for letter in letters {
        s.cmath(mathord, letter, letter, false);
        s.ctext(textord, letter, letter, false);
    }

    // Blackboard bold and script letters in Unicode range
    s.amath(textord, "C", "\u{2102}", false); // blackboard bold
    s.atext(textord, "C", "\u{2102}", false);
    s.amath(textord, "H", "\u{210D}", false);
    s.atext(textord, "H", "\u{210D}", false);
    s.amath(textord, "N", "\u{2115}", false);
    s.atext(textord, "N", "\u{2115}", false);
    s.amath(textord, "P", "\u{2119}", false);
    s.atext(textord, "P", "\u{2119}", false);
    s.amath(textord, "Q", "\u{211A}", false);
    s.atext(textord, "Q", "\u{211A}", false);
    s.amath(textord, "R", "\u{211D}", false);
    s.atext(textord, "R", "\u{211D}", false);
    s.amath(textord, "Z", "\u{2124}", false);
    s.atext(textord, "Z", "\u{2124}", false);
    s.cmath(mathord, "h", "\u{210E}", false); // italic h, Planck constant
    s.ctext(mathord, "h", "\u{210E}", false);

    // FIXME: Wide characters

    s
});

pub const LIGATURES: &'static [&'static str] = &["--", "---", "``", "''"];
