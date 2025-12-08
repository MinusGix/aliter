use std::{borrow::Cow, collections::HashMap};

use crate::{
    array::{AlignSpec, ColSeparationType},
    expander::Mode,
    lexer::Token,
    symbols::Atom,
    unit::Measurement,
    util::{SourceLocation, Style, StyleAuto},
};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseNode {
    Array(ArrayNode),
    CdLabel(CdLabelNode),
    CdLabelParentNode(CdLabelParentNode),
    Color(ColorNode),
    ColorToken(ColorTokenNode),
    Op(OpNode),
    OrdGroup(OrdGroupNode),
    Raw(RawNode),
    Size(SizeNode),
    Styling(StylingNode),
    SupSub(SupSubNode),
    Tag(TagNode),
    Text(TextNode),
    Url(UrlNode),
    Verb(VerbNode),
    Atom(AtomNode),
    MathOrd(MathOrdNode),
    Spacing(SpacingNode),
    TextOrd(TextOrdNode),
    AccentToken(AccentTokenNode),
    OpToken(OpTokenNode),
    Accent(AccentNode),
    AccentUnder(AccentUnderNode),
    Cr(CrNode),
    DelimSizing(DelimSizingNode),
    Enclose(EncloseNode),
    Environment(EnvironmentNode),
    Font(FontNode),
    GenFrac(GenFracNode),
    HBox(HBoxNode),
    HorizBrace(HorizBraceNode),
    Href(HrefNode),
    Html(HtmlNode),
    HtmlMathml(HtmlMathmlNode),
    IncludeGraphics(IncludeGraphicsNode),
    Infix(InfixNode),
    Internal(InternalNode),
    Kern(KernNode),
    Lap(LapNode),
    LeftRight(LeftRightNode),
    LeftRightRight(LeftRightRightNode),
    MathChoice(MathChoiceNode),
    Middle(MiddleNode),
    MClass(MClassNode),
    OperatorName(OperatorNameNode),
    Overline(OverlineNode),
    Phantom(PhantomNode),
    HPhantom(HPhantomNode),
    VPhantom(VPhantomNode),
    RaiseBox(RaiseBoxNode),
    Rule(RuleNode),
    Sizing(SizingNode),
    Smash(SmashNode),
    Sqrt(SqrtNode),
    Underline(UnderlineNode),
    VCenter(VCenterNode),
    XArrow(XArrowNode),
}
impl ParseNode {
    pub fn info(&self) -> &NodeInfo {
        match self {
            ParseNode::Array(a) => &a.info,
            ParseNode::CdLabel(a) => &a.info,
            ParseNode::CdLabelParentNode(a) => &a.info,
            ParseNode::Color(a) => &a.info,
            ParseNode::ColorToken(a) => &a.info,
            ParseNode::Op(a) => &a.info,
            ParseNode::OrdGroup(a) => &a.info,
            ParseNode::Raw(a) => &a.info,
            ParseNode::Size(a) => &a.info,
            ParseNode::Styling(a) => &a.info,
            ParseNode::SupSub(a) => &a.info,
            ParseNode::Tag(a) => &a.info,
            ParseNode::Text(a) => &a.info,
            ParseNode::Url(a) => &a.info,
            ParseNode::Verb(a) => &a.info,
            ParseNode::Atom(a) => &a.info,
            ParseNode::MathOrd(a) => &a.info,
            ParseNode::Spacing(a) => &a.info,
            ParseNode::TextOrd(a) => &a.info,
            ParseNode::AccentToken(a) => &a.info,
            ParseNode::OpToken(a) => &a.info,
            ParseNode::Accent(a) => &a.info,
            ParseNode::AccentUnder(a) => &a.info,
            ParseNode::Cr(a) => &a.info,
            ParseNode::DelimSizing(a) => &a.info,
            ParseNode::Enclose(a) => &a.info,
            ParseNode::Environment(a) => &a.info,
            ParseNode::Font(a) => &a.info,
            ParseNode::GenFrac(a) => &a.info,
            ParseNode::HBox(a) => &a.info,
            ParseNode::HorizBrace(a) => &a.info,
            ParseNode::Href(a) => &a.info,
            ParseNode::Html(a) => &a.info,
            ParseNode::HtmlMathml(a) => &a.info,
            ParseNode::IncludeGraphics(a) => &a.info,
            ParseNode::Infix(a) => &a.info,
            ParseNode::Internal(a) => &a.info,
            ParseNode::Kern(a) => &a.info,
            ParseNode::Lap(a) => &a.info,
            ParseNode::LeftRight(a) => &a.info,
            ParseNode::LeftRightRight(a) => &a.info,
            ParseNode::MathChoice(a) => &a.info,
            ParseNode::Middle(a) => &a.info,
            ParseNode::MClass(a) => &a.info,
            ParseNode::OperatorName(a) => &a.info,
            ParseNode::Overline(a) => &a.info,
            ParseNode::Phantom(a) => &a.info,
            ParseNode::HPhantom(a) => &a.info,
            ParseNode::VPhantom(a) => &a.info,
            ParseNode::RaiseBox(a) => &a.info,
            ParseNode::Rule(a) => &a.info,
            ParseNode::Sizing(a) => &a.info,
            ParseNode::Smash(a) => &a.info,
            ParseNode::Sqrt(a) => &a.info,
            ParseNode::Underline(a) => &a.info,
            ParseNode::VCenter(a) => &a.info,
            ParseNode::XArrow(a) => &a.info,
        }
    }

    pub fn info_mut(&mut self) -> &mut NodeInfo {
        match self {
            ParseNode::Array(a) => &mut a.info,
            ParseNode::CdLabel(a) => &mut a.info,
            ParseNode::CdLabelParentNode(a) => &mut a.info,
            ParseNode::Color(a) => &mut a.info,
            ParseNode::ColorToken(a) => &mut a.info,
            ParseNode::Op(a) => &mut a.info,
            ParseNode::OrdGroup(a) => &mut a.info,
            ParseNode::Raw(a) => &mut a.info,
            ParseNode::Size(a) => &mut a.info,
            ParseNode::Styling(a) => &mut a.info,
            ParseNode::SupSub(a) => &mut a.info,
            ParseNode::Tag(a) => &mut a.info,
            ParseNode::Text(a) => &mut a.info,
            ParseNode::Url(a) => &mut a.info,
            ParseNode::Verb(a) => &mut a.info,
            ParseNode::Atom(a) => &mut a.info,
            ParseNode::MathOrd(a) => &mut a.info,
            ParseNode::Spacing(a) => &mut a.info,
            ParseNode::TextOrd(a) => &mut a.info,
            ParseNode::AccentToken(a) => &mut a.info,
            ParseNode::OpToken(a) => &mut a.info,
            ParseNode::Accent(a) => &mut a.info,
            ParseNode::AccentUnder(a) => &mut a.info,
            ParseNode::Cr(a) => &mut a.info,
            ParseNode::DelimSizing(a) => &mut a.info,
            ParseNode::Enclose(a) => &mut a.info,
            ParseNode::Environment(a) => &mut a.info,
            ParseNode::Font(a) => &mut a.info,
            ParseNode::GenFrac(a) => &mut a.info,
            ParseNode::HBox(a) => &mut a.info,
            ParseNode::HorizBrace(a) => &mut a.info,
            ParseNode::Href(a) => &mut a.info,
            ParseNode::Html(a) => &mut a.info,
            ParseNode::HtmlMathml(a) => &mut a.info,
            ParseNode::IncludeGraphics(a) => &mut a.info,
            ParseNode::Infix(a) => &mut a.info,
            ParseNode::Internal(a) => &mut a.info,
            ParseNode::Kern(a) => &mut a.info,
            ParseNode::Lap(a) => &mut a.info,
            ParseNode::LeftRight(a) => &mut a.info,
            ParseNode::LeftRightRight(a) => &mut a.info,
            ParseNode::MathChoice(a) => &mut a.info,
            ParseNode::Middle(a) => &mut a.info,
            ParseNode::MClass(a) => &mut a.info,
            ParseNode::OperatorName(a) => &mut a.info,
            ParseNode::Overline(a) => &mut a.info,
            ParseNode::Phantom(a) => &mut a.info,
            ParseNode::HPhantom(a) => &mut a.info,
            ParseNode::VPhantom(a) => &mut a.info,
            ParseNode::RaiseBox(a) => &mut a.info,
            ParseNode::Rule(a) => &mut a.info,
            ParseNode::Sizing(a) => &mut a.info,
            ParseNode::Smash(a) => &mut a.info,
            ParseNode::Sqrt(a) => &mut a.info,
            ParseNode::Underline(a) => &mut a.info,
            ParseNode::VCenter(a) => &mut a.info,
            ParseNode::XArrow(a) => &mut a.info,
        }
    }

    pub fn loc(&self) -> Option<SourceLocation> {
        self.info().loc.clone()
    }

    pub fn text(&self) -> Option<&str> {
        match self {
            ParseNode::Atom(a) => Some(&a.text),
            ParseNode::MathOrd(a) => Some(&a.text),
            ParseNode::Spacing(a) => Some(&a.text),
            ParseNode::TextOrd(a) => Some(&a.text),
            ParseNode::AccentToken(a) => Some(&a.text),
            ParseNode::OpToken(a) => Some(&a.text),
            _ => None,
        }
    }

    pub fn typ(&self) -> ParseNodeType {
        match self {
            ParseNode::Array(_) => ParseNodeType::Array,
            ParseNode::CdLabel(_) => ParseNodeType::CdLabel,
            ParseNode::CdLabelParentNode(_) => ParseNodeType::CdLabelParentNode,
            ParseNode::Color(_) => ParseNodeType::Color,
            ParseNode::ColorToken(_) => ParseNodeType::ColorToken,
            ParseNode::Op(_) => ParseNodeType::Op,
            ParseNode::OrdGroup(_) => ParseNodeType::OrdGroup,
            ParseNode::Raw(_) => ParseNodeType::Raw,
            ParseNode::Size(_) => ParseNodeType::Size,
            ParseNode::Styling(_) => ParseNodeType::Styling,
            ParseNode::SupSub(_) => ParseNodeType::SupSub,
            ParseNode::Tag(_) => ParseNodeType::Tag,
            ParseNode::Text(_) => ParseNodeType::Text,
            ParseNode::Url(_) => ParseNodeType::Url,
            ParseNode::Verb(_) => ParseNodeType::Verb,
            ParseNode::Atom(_) => ParseNodeType::Atom,
            ParseNode::MathOrd(_) => ParseNodeType::MathOrd,
            ParseNode::Spacing(_) => ParseNodeType::Spacing,
            ParseNode::TextOrd(_) => ParseNodeType::TextOrd,
            ParseNode::AccentToken(_) => ParseNodeType::AccentToken,
            ParseNode::OpToken(_) => ParseNodeType::OpToken,
            ParseNode::Accent(_) => ParseNodeType::Accent,
            ParseNode::AccentUnder(_) => ParseNodeType::AccentUnder,
            ParseNode::Cr(_) => ParseNodeType::Cr,
            ParseNode::DelimSizing(_) => ParseNodeType::DelimSizing,
            ParseNode::Enclose(_) => ParseNodeType::Enclose,
            ParseNode::Environment(_) => ParseNodeType::Environment,
            ParseNode::Font(_) => ParseNodeType::Font,
            ParseNode::GenFrac(_) => ParseNodeType::GenFrac,
            ParseNode::HBox(_) => ParseNodeType::HBox,
            ParseNode::HorizBrace(_) => ParseNodeType::HorizBrace,
            ParseNode::Href(_) => ParseNodeType::Href,
            ParseNode::Html(_) => ParseNodeType::Html,
            ParseNode::HtmlMathml(_) => ParseNodeType::HtmlMathml,
            ParseNode::IncludeGraphics(_) => ParseNodeType::IncludeGraphics,
            ParseNode::Infix(_) => ParseNodeType::Infix,
            ParseNode::Internal(_) => ParseNodeType::Internal,
            ParseNode::Kern(_) => ParseNodeType::Kern,
            ParseNode::Lap(_) => ParseNodeType::Lap,
            ParseNode::LeftRight(_) => ParseNodeType::LeftRight,
            ParseNode::LeftRightRight(_) => ParseNodeType::LeftRightRight,
            ParseNode::MathChoice(_) => ParseNodeType::MathChoice,
            ParseNode::Middle(_) => ParseNodeType::Middle,
            ParseNode::MClass(_) => ParseNodeType::MClass,
            ParseNode::OperatorName(_) => ParseNodeType::OperatorName,
            ParseNode::Overline(_) => ParseNodeType::Overline,
            ParseNode::Phantom(_) => ParseNodeType::Phantom,
            ParseNode::HPhantom(_) => ParseNodeType::HPhantom,
            ParseNode::VPhantom(_) => ParseNodeType::VPhantomNode,
            ParseNode::RaiseBox(_) => ParseNodeType::RaiseBox,
            ParseNode::Rule(_) => ParseNodeType::Rule,
            ParseNode::Sizing(_) => ParseNodeType::Sizing,
            ParseNode::Smash(_) => ParseNodeType::Smash,
            ParseNode::Sqrt(_) => ParseNodeType::Sqrt,
            ParseNode::Underline(_) => ParseNodeType::Underline,
            ParseNode::VCenter(_) => ParseNodeType::VCenter,
            ParseNode::XArrow(_) => ParseNodeType::XArrow,
        }
    }
}
pub trait EqNoLoc {
    /// Returns true if the two nodes are equal, ignoring their locations.
    fn eq_no_loc(&self, o: &Self) -> bool;
}
impl<T: EqNoLoc> EqNoLoc for Vec<T> {
    fn eq_no_loc(&self, o: &Self) -> bool {
        if self.len() != o.len() {
            return false;
        }

        for (a, b) in self.iter().zip(o.iter()) {
            if !a.eq_no_loc(b) {
                return false;
            }
        }

        true
    }
}
impl<T: EqNoLoc> EqNoLoc for Option<T> {
    fn eq_no_loc(&self, o: &Self) -> bool {
        match (self, o) {
            (Some(a), Some(b)) => a.eq_no_loc(b),
            (None, None) => true,
            _ => false,
        }
    }
}
impl<'a> EqNoLoc for &'a ParseNode {
    fn eq_no_loc(&self, o: &Self) -> bool {
        (*self).eq_no_loc(*o)
    }
}
impl EqNoLoc for ParseNode {
    fn eq_no_loc(&self, o: &Self) -> bool {
        match (self, o) {
            (ParseNode::Array(a), ParseNode::Array(b)) => a.eq_no_loc(b),
            (ParseNode::CdLabel(a), ParseNode::CdLabel(b)) => a.eq_no_loc(b),
            (ParseNode::CdLabelParentNode(a), ParseNode::CdLabelParentNode(b)) => a.eq_no_loc(b),
            (ParseNode::Color(a), ParseNode::Color(b)) => a.eq_no_loc(b),
            (ParseNode::ColorToken(a), ParseNode::ColorToken(b)) => a.eq_no_loc(b),
            (ParseNode::Op(a), ParseNode::Op(b)) => a.eq_no_loc(b),
            (ParseNode::OrdGroup(a), ParseNode::OrdGroup(b)) => a.eq_no_loc(b),
            (ParseNode::Raw(a), ParseNode::Raw(b)) => a.eq_no_loc(b),
            (ParseNode::Size(a), ParseNode::Size(b)) => a.eq_no_loc(b),
            (ParseNode::Styling(a), ParseNode::Styling(b)) => a.eq_no_loc(b),
            (ParseNode::SupSub(a), ParseNode::SupSub(b)) => a.eq_no_loc(b),
            (ParseNode::Tag(a), ParseNode::Tag(b)) => a.eq_no_loc(b),
            (ParseNode::Text(a), ParseNode::Text(b)) => a.eq_no_loc(b),
            (ParseNode::Url(a), ParseNode::Url(b)) => a.eq_no_loc(b),
            (ParseNode::Verb(a), ParseNode::Verb(b)) => a.eq_no_loc(b),
            (ParseNode::Atom(a), ParseNode::Atom(b)) => a.eq_no_loc(b),
            (ParseNode::MathOrd(a), ParseNode::MathOrd(b)) => a.eq_no_loc(b),
            (ParseNode::Spacing(a), ParseNode::Spacing(b)) => a.eq_no_loc(b),
            (ParseNode::TextOrd(a), ParseNode::TextOrd(b)) => a.eq_no_loc(b),
            (ParseNode::AccentToken(a), ParseNode::AccentToken(b)) => a.eq_no_loc(b),
            (ParseNode::OpToken(a), ParseNode::OpToken(b)) => a.eq_no_loc(b),
            (ParseNode::Accent(a), ParseNode::Accent(b)) => a.eq_no_loc(b),
            (ParseNode::AccentUnder(a), ParseNode::AccentUnder(b)) => a.eq_no_loc(b),
            (ParseNode::Cr(a), ParseNode::Cr(b)) => a.eq_no_loc(b),
            (ParseNode::DelimSizing(a), ParseNode::DelimSizing(b)) => a.eq_no_loc(b),
            (ParseNode::Enclose(a), ParseNode::Enclose(b)) => a.eq_no_loc(b),
            (ParseNode::Environment(a), ParseNode::Environment(b)) => a.eq_no_loc(b),
            (ParseNode::Font(a), ParseNode::Font(b)) => a.eq_no_loc(b),
            (ParseNode::GenFrac(a), ParseNode::GenFrac(b)) => a.eq_no_loc(b),
            (ParseNode::HBox(a), ParseNode::HBox(b)) => a.eq_no_loc(b),
            (ParseNode::HorizBrace(a), ParseNode::HorizBrace(b)) => a.eq_no_loc(b),
            (ParseNode::Href(a), ParseNode::Href(b)) => a.eq_no_loc(b),
            (ParseNode::Html(a), ParseNode::Html(b)) => a.eq_no_loc(b),
            (ParseNode::HtmlMathml(a), ParseNode::HtmlMathml(b)) => a.eq_no_loc(b),
            (ParseNode::IncludeGraphics(a), ParseNode::IncludeGraphics(b)) => a.eq_no_loc(b),
            (ParseNode::Infix(a), ParseNode::Infix(b)) => a.eq_no_loc(b),
            (ParseNode::Internal(a), ParseNode::Internal(b)) => a.eq_no_loc(b),
            (ParseNode::Kern(a), ParseNode::Kern(b)) => a.eq_no_loc(b),
            (ParseNode::Lap(a), ParseNode::Lap(b)) => a.eq_no_loc(b),
            (ParseNode::LeftRight(a), ParseNode::LeftRight(b)) => a.eq_no_loc(b),
            (ParseNode::LeftRightRight(a), ParseNode::LeftRightRight(b)) => a.eq_no_loc(b),
            (ParseNode::MathChoice(a), ParseNode::MathChoice(b)) => a.eq_no_loc(b),
            (ParseNode::Middle(a), ParseNode::Middle(b)) => a.eq_no_loc(b),
            (ParseNode::MClass(a), ParseNode::MClass(b)) => a.eq_no_loc(b),
            (ParseNode::OperatorName(a), ParseNode::OperatorName(b)) => a.eq_no_loc(b),
            (ParseNode::Overline(a), ParseNode::Overline(b)) => a.eq_no_loc(b),
            (ParseNode::Phantom(a), ParseNode::Phantom(b)) => a.eq_no_loc(b),
            (ParseNode::HPhantom(a), ParseNode::HPhantom(b)) => a.eq_no_loc(b),
            (ParseNode::VPhantom(a), ParseNode::VPhantom(b)) => a.eq_no_loc(b),
            (ParseNode::RaiseBox(a), ParseNode::RaiseBox(b)) => a.eq_no_loc(b),
            (ParseNode::Rule(a), ParseNode::Rule(b)) => a.eq_no_loc(b),
            (ParseNode::Sizing(a), ParseNode::Sizing(b)) => a.eq_no_loc(b),
            (ParseNode::Smash(a), ParseNode::Smash(b)) => a.eq_no_loc(b),
            (ParseNode::Sqrt(a), ParseNode::Sqrt(b)) => a.eq_no_loc(b),
            (ParseNode::Underline(a), ParseNode::Underline(b)) => a.eq_no_loc(b),
            (ParseNode::VCenter(a), ParseNode::VCenter(b)) => a.eq_no_loc(b),
            (ParseNode::XArrow(a), ParseNode::XArrow(b)) => a.eq_no_loc(b),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseNodeType {
    Array,
    CdLabel,
    CdLabelParentNode,
    Color,
    ColorToken,
    Op,
    OrdGroup,
    Raw,
    Size,
    Styling,
    SupSub,
    Tag,
    Text,
    Url,
    Verb,
    Atom,
    MathOrd,
    Spacing,
    TextOrd,
    AccentToken,
    OpToken,
    Accent,
    AccentUnder,
    Cr,
    DelimSizing,
    Enclose,
    Environment,
    Font,
    GenFrac,
    HBox,
    HorizBrace,
    Href,
    Html,
    HtmlMathml,
    IncludeGraphics,
    Infix,
    Internal,
    Kern,
    Lap,
    LeftRight,
    LeftRightRight,
    MathChoice,
    Middle,
    MClass,
    OperatorName,
    Overline,
    Phantom,
    HPhantom,
    VPhantomNode,
    RaiseBox,
    Rule,
    Sizing,
    Smash,
    Sqrt,
    Underline,
    VCenter,
    XArrow,
}

pub trait SymbolParseNode {
    fn text(&self) -> &str;

    fn info(&self) -> &NodeInfo;
}
impl SymbolParseNode for AtomNode {
    fn text(&self) -> &str {
        &self.text
    }

    fn info(&self) -> &NodeInfo {
        &self.info
    }
}
impl SymbolParseNode for AccentTokenNode {
    fn text(&self) -> &str {
        &self.text
    }

    fn info(&self) -> &NodeInfo {
        &self.info
    }
}
impl SymbolParseNode for MathOrdNode {
    fn text(&self) -> &str {
        &self.text
    }

    fn info(&self) -> &NodeInfo {
        &self.info
    }
}
impl SymbolParseNode for OpTokenNode {
    fn text(&self) -> &str {
        &self.text
    }

    fn info(&self) -> &NodeInfo {
        &self.info
    }
}
impl SymbolParseNode for SpacingNode {
    fn text(&self) -> &str {
        &self.text
    }

    fn info(&self) -> &NodeInfo {
        &self.info
    }
}
impl SymbolParseNode for TextOrdNode {
    fn text(&self) -> &str {
        &self.text
    }

    fn info(&self) -> &NodeInfo {
        &self.info
    }
}

pub type UnsupportedCmdParseNode = ColorNode;

#[derive(Debug, Clone, PartialEq)]
pub struct NodeInfo {
    pub mode: Mode,
    pub loc: Option<SourceLocation>,
}
impl NodeInfo {
    pub fn new_mode(mode: Mode) -> NodeInfo {
        NodeInfo { mode, loc: None }
    }
}
impl EqNoLoc for NodeInfo {
    fn eq_no_loc(&self, o: &Self) -> bool {
        self.mode == o.mode
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayNode {
    pub col_separation_type: Option<ColSeparationType>,
    pub h_skip_before_and_after: Option<bool>,
    pub add_jot: Option<bool>,
    pub cols: Option<Vec<AlignSpec>>,
    pub array_stretch: usize,
    // TODO: body?
    pub row_gaps: Vec<Option<Measurement>>,
    pub h_lines_before_row: Vec<Vec<bool>>,
    /// Whether each row should be automatically number or an explicit tag
    // TODO: pub tags:
    pub leq_no: Option<bool>,
    pub is_cd: Option<bool>,
    pub info: NodeInfo,
}
impl EqNoLoc for ArrayNode {
    fn eq_no_loc(&self, o: &Self) -> bool {
        self.col_separation_type == o.col_separation_type
            && self.h_skip_before_and_after == o.h_skip_before_and_after
            && self.add_jot == o.add_jot
            && self.cols == o.cols
            && self.array_stretch == o.array_stretch
            && self.row_gaps == o.row_gaps
            && self.h_lines_before_row == o.h_lines_before_row
            && self.leq_no == o.leq_no
            && self.is_cd == o.is_cd
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CdLabelNode {
    pub side: Cow<'static, str>,
    pub label: Box<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for CdLabelNode {
    fn eq_no_loc(&self, o: &CdLabelNode) -> bool {
        self.side == o.side && self.label.eq_no_loc(&o.label) && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CdLabelParentNode {
    pub fragment: Box<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for CdLabelParentNode {
    fn eq_no_loc(&self, o: &CdLabelParentNode) -> bool {
        self.fragment.eq_no_loc(&o.fragment) && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorNode {
    pub color: Color,
    pub info: NodeInfo,
    pub body: Vec<ParseNode>,
}
impl EqNoLoc for ColorNode {
    fn eq_no_loc(&self, o: &ColorNode) -> bool {
        self.color == o.color && self.info.eq_no_loc(&o.info) && self.body.eq_no_loc(&o.body)
    }
}
/// Note: The [`PartialEq`] impl does not consider equivalencies like
/// `RGB([5, 9, 2]) == RGBA([5, 9, 2, 0xff])`
#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    RGB([u8; 3]),
    RGBA([u8; 4]),
    Named(Cow<'static, str>),
}
impl Color {
    pub(crate) fn to_string(&self) -> String {
        match self {
            Color::RGB(rgb) => format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2]),
            Color::RGBA(rgba) => format!(
                "#{:02x}{:02x}{:02x}{:02x}",
                rgba[0], rgba[1], rgba[2], rgba[3]
            ),
            Color::Named(name) => name.clone().into_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorTokenNode {
    pub color: Color,
    pub info: NodeInfo,
}
impl EqNoLoc for ColorTokenNode {
    fn eq_no_loc(&self, o: &ColorTokenNode) -> bool {
        self.color == o.color && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpNode {
    pub limits: bool,
    pub always_handle_sup_sub: Option<bool>,
    pub suppress_base_shift: Option<bool>,
    pub parent_is_sup_sub: Option<bool>,
    /// If symbol is true, then `body` must be set
    pub symbol: bool,
    // Note: The docs say that body and value are never set simultaneously
    // however there is no field name valued
    // However the code shows that `name` is nothing when `body` is something
    // so I presume that is what it means
    pub name: Option<Cow<'static, str>>,
    pub body: Option<Vec<ParseNode>>,
    pub info: NodeInfo,
}
impl EqNoLoc for OpNode {
    fn eq_no_loc(&self, o: &OpNode) -> bool {
        self.limits == o.limits
            && self.always_handle_sup_sub == o.always_handle_sup_sub
            && self.suppress_base_shift == o.suppress_base_shift
            && self.parent_is_sup_sub == o.parent_is_sup_sub
            && self.symbol == o.symbol
            && self.name == o.name
            && self.body == o.body
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrdGroupNode {
    pub body: Vec<ParseNode>,
    pub semi_simple: Option<bool>,
    pub info: NodeInfo,
}
impl EqNoLoc for OrdGroupNode {
    fn eq_no_loc(&self, o: &OrdGroupNode) -> bool {
        self.body.eq_no_loc(&o.body)
            && self.semi_simple == o.semi_simple
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawNode {
    pub string: String,
    pub info: NodeInfo,
}
impl EqNoLoc for RawNode {
    fn eq_no_loc(&self, o: &RawNode) -> bool {
        self.string == o.string && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SizeNode {
    pub value: Measurement,
    pub is_blank: bool,
    pub info: NodeInfo,
}
impl EqNoLoc for SizeNode {
    fn eq_no_loc(&self, o: &SizeNode) -> bool {
        self.value == o.value && self.is_blank == o.is_blank && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StylingNode {
    pub style: Style,
    pub body: Vec<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for StylingNode {
    fn eq_no_loc(&self, o: &StylingNode) -> bool {
        self.style == o.style && self.body.eq_no_loc(&o.body) && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SupSubNode {
    pub base: Option<Box<ParseNode>>,
    pub sup: Option<Box<ParseNode>>,
    pub sub: Option<Box<ParseNode>>,
    pub info: NodeInfo,
}
impl EqNoLoc for SupSubNode {
    fn eq_no_loc(&self, o: &SupSubNode) -> bool {
        self.base.as_deref().eq_no_loc(&o.base.as_deref())
            && self.sup.as_deref().eq_no_loc(&o.sup.as_deref())
            && self.sub.as_deref().eq_no_loc(&o.sub.as_deref())
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TagNode {
    pub body: Vec<ParseNode>,
    pub tag: Vec<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for TagNode {
    fn eq_no_loc(&self, o: &TagNode) -> bool {
        self.body.eq_no_loc(&o.body) && self.tag.eq_no_loc(&o.tag) && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextNode {
    pub body: Vec<ParseNode>,
    pub font: Option<Cow<'static, str>>,
    pub info: NodeInfo,
}
impl EqNoLoc for TextNode {
    fn eq_no_loc(&self, o: &TextNode) -> bool {
        self.body.eq_no_loc(&o.body) && self.font == o.font && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UrlNode {
    pub url: String,
    pub info: NodeInfo,
}
impl EqNoLoc for UrlNode {
    fn eq_no_loc(&self, o: &UrlNode) -> bool {
        self.url == o.url && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerbNode {
    pub body: Cow<'static, str>,
    pub star: bool,
    pub info: NodeInfo,
}
impl EqNoLoc for VerbNode {
    fn eq_no_loc(&self, o: &VerbNode) -> bool {
        self.body == o.body && self.star == o.star && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtomNode {
    pub family: Atom,
    pub text: String,
    pub info: NodeInfo,
}
impl EqNoLoc for AtomNode {
    fn eq_no_loc(&self, o: &AtomNode) -> bool {
        self.family == o.family && self.text == o.text && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MathOrdNode {
    pub text: String,
    pub info: NodeInfo,
}
impl EqNoLoc for MathOrdNode {
    fn eq_no_loc(&self, o: &MathOrdNode) -> bool {
        self.text == o.text && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpacingNode {
    pub text: String,
    pub info: NodeInfo,
}
impl EqNoLoc for SpacingNode {
    fn eq_no_loc(&self, o: &SpacingNode) -> bool {
        self.text == o.text && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextOrdNode {
    pub text: Cow<'static, str>,
    pub info: NodeInfo,
}
impl EqNoLoc for TextOrdNode {
    fn eq_no_loc(&self, o: &TextOrdNode) -> bool {
        self.text == o.text && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccentTokenNode {
    pub text: String,
    pub info: NodeInfo,
}
impl EqNoLoc for AccentTokenNode {
    fn eq_no_loc(&self, o: &AccentTokenNode) -> bool {
        self.text == o.text && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpTokenNode {
    pub text: String,
    pub info: NodeInfo,
}
impl EqNoLoc for OpTokenNode {
    fn eq_no_loc(&self, o: &OpTokenNode) -> bool {
        self.text == o.text && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccentNode {
    pub label: Cow<'static, str>,
    pub is_stretchy: Option<bool>,
    pub is_shifty: Option<bool>,
    pub base: Box<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for AccentNode {
    fn eq_no_loc(&self, o: &AccentNode) -> bool {
        self.label == o.label
            && self.is_stretchy == o.is_stretchy
            && self.is_shifty == o.is_shifty
            && self.base.as_ref().eq_no_loc(o.base.as_ref())
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccentUnderNode {
    pub label: String,
    pub is_stretchy: Option<bool>,
    pub is_shifty: Option<bool>,
    pub base: Box<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for AccentUnderNode {
    fn eq_no_loc(&self, o: &AccentUnderNode) -> bool {
        self.label == o.label
            && self.is_stretchy == o.is_stretchy
            && self.is_shifty == o.is_shifty
            && self.base.as_ref().eq_no_loc(o.base.as_ref())
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CrNode {
    pub new_line: bool,
    pub size: Option<Measurement>,
    pub info: NodeInfo,
}
impl EqNoLoc for CrNode {
    fn eq_no_loc(&self, o: &CrNode) -> bool {
        self.new_line == o.new_line && self.size == o.size && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum DelimSize {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MClass {
    Open,
    Close,
    Rel,
    Ord,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelimSizingNode {
    pub size: DelimSize,
    pub m_class: MClass,
    pub delim: Cow<'static, str>,
    pub info: NodeInfo,
}
impl EqNoLoc for DelimSizingNode {
    fn eq_no_loc(&self, o: &Self) -> bool {
        self.size == o.size
            && self.m_class == o.m_class
            && self.delim == o.delim
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EncloseNode {
    pub label: String,
    // TODO: Should this be a more general color
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for EncloseNode {
    fn eq_no_loc(&self, o: &EncloseNode) -> bool {
        self.label == o.label
            && self.background_color == o.background_color
            && self.border_color == o.border_color
            && self.body.eq_no_loc(&o.body)
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnvironmentNode {
    pub name: String,
    pub name_group: Box<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for EnvironmentNode {
    fn eq_no_loc(&self, o: &EnvironmentNode) -> bool {
        self.name == o.name
            && self.name_group.as_ref().eq_no_loc(o.name_group.as_ref())
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FontNode {
    pub font: Cow<'static, str>,
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for FontNode {
    fn eq_no_loc(&self, o: &FontNode) -> bool {
        self.font == o.font && self.body.eq_no_loc(&o.body) && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenFracNode {
    pub continued: bool,
    pub numer: Box<ParseNode>,
    pub denom: Box<ParseNode>,
    pub has_bar_line: bool,
    pub left_delim: Option<Cow<'static, str>>,
    pub right_delim: Option<Cow<'static, str>>,
    pub size: StyleAuto,
    pub bar_size: Option<Measurement>,
    pub info: NodeInfo,
}
impl EqNoLoc for GenFracNode {
    fn eq_no_loc(&self, o: &GenFracNode) -> bool {
        self.continued == o.continued
            && self.numer.as_ref().eq_no_loc(o.numer.as_ref())
            && self.denom.as_ref().eq_no_loc(o.denom.as_ref())
            && self.has_bar_line == o.has_bar_line
            && self.left_delim == o.left_delim
            && self.right_delim == o.right_delim
            && self.size == o.size
            && self.bar_size == o.bar_size
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HBoxNode {
    pub body: Vec<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for HBoxNode {
    fn eq_no_loc(&self, o: &HBoxNode) -> bool {
        self.body.eq_no_loc(&o.body) && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HorizBraceNode {
    pub label: String,
    pub is_over: bool,
    pub base: Box<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for HorizBraceNode {
    fn eq_no_loc(&self, o: &HorizBraceNode) -> bool {
        self.label == o.label
            && self.is_over == o.is_over
            && self.base.as_ref().eq_no_loc(o.base.as_ref())
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HrefNode {
    pub href: String,
    pub body: Vec<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for HrefNode {
    fn eq_no_loc(&self, o: &HrefNode) -> bool {
        self.href == o.href && self.body.eq_no_loc(&o.body) && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HtmlNode {
    pub attributes: HashMap<String, String>,
    pub body: Vec<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for HtmlNode {
    fn eq_no_loc(&self, o: &HtmlNode) -> bool {
        self.attributes == o.attributes
            && self.body.eq_no_loc(&o.body)
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HtmlMathmlNode {
    pub html: Vec<ParseNode>,
    pub mathml: Vec<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for HtmlMathmlNode {
    fn eq_no_loc(&self, o: &HtmlMathmlNode) -> bool {
        self.html.eq_no_loc(&o.html)
            && self.mathml.eq_no_loc(&o.mathml)
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IncludeGraphicsNode {
    pub alt: String,
    pub width: Measurement,
    pub height: Measurement,
    pub total_height: Measurement,
    pub src: String,
    pub info: NodeInfo,
}
impl EqNoLoc for IncludeGraphicsNode {
    fn eq_no_loc(&self, o: &IncludeGraphicsNode) -> bool {
        self.alt == o.alt
            && self.width == o.width
            && self.height == o.height
            && self.total_height == o.total_height
            && self.src == o.src
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InfixNode {
    pub replace_with: Cow<'static, str>,
    pub size: Option<Measurement>,
    pub token: Option<Token<'static>>,
    pub info: NodeInfo,
}
impl EqNoLoc for InfixNode {
    fn eq_no_loc(&self, o: &InfixNode) -> bool {
        self.replace_with == o.replace_with
            && self.size == o.size
            && self.token == o.token
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InternalNode {
    pub info: NodeInfo,
}
impl EqNoLoc for InternalNode {
    fn eq_no_loc(&self, o: &InternalNode) -> bool {
        self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KernNode {
    pub dimension: Measurement,
    pub info: NodeInfo,
}
impl EqNoLoc for KernNode {
    fn eq_no_loc(&self, o: &KernNode) -> bool {
        self.dimension == o.dimension && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LapNode {
    pub alignment: String,
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for LapNode {
    fn eq_no_loc(&self, o: &LapNode) -> bool {
        self.alignment == o.alignment
            && self.body.eq_no_loc(&o.body)
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeftRightNode {
    pub left: String,
    pub right: String,
    pub right_color: Option<Color>,
    pub body: Vec<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for LeftRightNode {
    fn eq_no_loc(&self, o: &LeftRightNode) -> bool {
        self.left == o.left
            && self.right == o.right
            && self.right_color == o.right_color
            && self.body.eq_no_loc(&o.body)
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeftRightRightNode {
    pub delim: String,
    pub color: Option<Color>,
    pub info: NodeInfo,
}
impl EqNoLoc for LeftRightRightNode {
    fn eq_no_loc(&self, o: &LeftRightRightNode) -> bool {
        self.delim == o.delim && self.color == o.color && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MathChoiceNode {
    pub display: Vec<ParseNode>,
    pub text: Vec<ParseNode>,
    pub script: Vec<ParseNode>,
    pub script_script: Vec<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for MathChoiceNode {
    fn eq_no_loc(&self, o: &MathChoiceNode) -> bool {
        self.display.eq_no_loc(&o.display)
            && self.text.eq_no_loc(&o.text)
            && self.script.eq_no_loc(&o.script)
            && self.script_script.eq_no_loc(&o.script_script)
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MiddleNode {
    pub delim: String,
    pub info: NodeInfo,
}
impl EqNoLoc for MiddleNode {
    fn eq_no_loc(&self, o: &MiddleNode) -> bool {
        self.delim == o.delim && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MClassNode {
    pub m_class: String,
    pub body: Vec<ParseNode>,
    pub is_character_box: bool,
    pub info: NodeInfo,
}
impl EqNoLoc for MClassNode {
    fn eq_no_loc(&self, o: &MClassNode) -> bool {
        self.m_class == o.m_class
            && self.body.eq_no_loc(&o.body)
            && self.is_character_box == o.is_character_box
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperatorNameNode {
    pub body: Vec<ParseNode>,
    pub always_handle_sup_sub: bool,
    pub limits: bool,
    pub parent_is_sup_sub: bool,
    pub info: NodeInfo,
}
impl EqNoLoc for OperatorNameNode {
    fn eq_no_loc(&self, o: &OperatorNameNode) -> bool {
        self.body.eq_no_loc(&o.body)
            && self.always_handle_sup_sub == o.always_handle_sup_sub
            && self.limits == o.limits
            && self.parent_is_sup_sub == o.parent_is_sup_sub
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlineNode {
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for OverlineNode {
    fn eq_no_loc(&self, o: &OverlineNode) -> bool {
        self.body.eq_no_loc(&o.body) && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhantomNode {
    pub body: Vec<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for PhantomNode {
    fn eq_no_loc(&self, o: &PhantomNode) -> bool {
        self.body.eq_no_loc(&o.body) && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HPhantomNode {
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for HPhantomNode {
    fn eq_no_loc(&self, o: &HPhantomNode) -> bool {
        self.body.eq_no_loc(&o.body) && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VPhantomNode {
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for VPhantomNode {
    fn eq_no_loc(&self, o: &VPhantomNode) -> bool {
        self.body.eq_no_loc(&o.body) && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RaiseBoxNode {
    pub dy: Measurement,
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for RaiseBoxNode {
    fn eq_no_loc(&self, o: &RaiseBoxNode) -> bool {
        self.dy == o.dy && self.body.eq_no_loc(&o.body) && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleNode {
    pub shift: Option<Measurement>,
    pub width: Measurement,
    pub height: Measurement,
    pub info: NodeInfo,
}
impl EqNoLoc for RuleNode {
    fn eq_no_loc(&self, o: &RuleNode) -> bool {
        self.shift == o.shift
            && self.width == o.width
            && self.height == o.height
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SizingNode {
    // TODO: floating point?
    pub size: usize,
    pub body: Vec<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for SizingNode {
    fn eq_no_loc(&self, o: &SizingNode) -> bool {
        self.size == o.size && self.body.eq_no_loc(&o.body) && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SmashNode {
    pub body: Box<ParseNode>,
    pub smash_height: bool,
    pub smash_depth: bool,
    pub info: NodeInfo,
}
impl EqNoLoc for SmashNode {
    fn eq_no_loc(&self, o: &SmashNode) -> bool {
        self.body.eq_no_loc(&o.body)
            && self.smash_height == o.smash_height
            && self.smash_depth == o.smash_depth
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SqrtNode {
    pub body: Box<ParseNode>,
    pub index: Option<Box<ParseNode>>,
    pub info: NodeInfo,
}
impl EqNoLoc for SqrtNode {
    fn eq_no_loc(&self, o: &SqrtNode) -> bool {
        self.body.eq_no_loc(&o.body)
            && self.index.as_deref().eq_no_loc(&o.index.as_deref())
            && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnderlineNode {
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for UnderlineNode {
    fn eq_no_loc(&self, o: &UnderlineNode) -> bool {
        self.body.eq_no_loc(&o.body) && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VCenterNode {
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}
impl EqNoLoc for VCenterNode {
    fn eq_no_loc(&self, o: &VCenterNode) -> bool {
        self.body.eq_no_loc(&o.body) && self.info.eq_no_loc(&o.info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct XArrowNode {
    pub label: String,
    pub body: Box<ParseNode>,
    pub below: Option<Box<ParseNode>>,
    pub info: NodeInfo,
}
impl EqNoLoc for XArrowNode {
    fn eq_no_loc(&self, o: &XArrowNode) -> bool {
        self.label == o.label
            && self.body.eq_no_loc(&o.body)
            && self.below.as_deref().eq_no_loc(&o.below.as_deref())
            && self.info.eq_no_loc(&o.info)
    }
}
