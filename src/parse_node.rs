use std::{borrow::Cow, collections::HashMap};

use crate::{
    array::{AlignSpec, ColSeparationType},
    expander::Mode,
    lexer::Token,
    symbols::Atom,
    unit::Measurement,
    util::{SourceLocation, Style, StyleAuto},
};

#[derive(Debug, Clone)]
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
    HtmlMathML(HtmlMathMLNode),
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
    VPhantomNode(VPhantomNode),
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
            ParseNode::HtmlMathML(a) => &a.info,
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
            ParseNode::VPhantomNode(a) => &a.info,
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
    HtmlMathML,
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

#[derive(Debug, Clone)]
pub enum SymbolParseNode {
    Atom(AtomNode),
    AccentToken(AccentTokenNode),
    MathOrd(MathOrdNode),
    OpToken(OpTokenNode),
    Spacing(SpacingNode),
    TextOrd(TextOrdNode),
}

pub type UnsupportedCmdParseNode = ColorNode;

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub mode: Mode,
    pub loc: Option<SourceLocation>,
}
impl NodeInfo {
    pub fn new_mode(mode: Mode) -> NodeInfo {
        NodeInfo { mode, loc: None }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct CdLabelNode {
    pub side: Cow<'static, str>,
    pub label: Box<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct CdLabelParentNode {
    pub fragment: Box<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct ColorNode {
    pub color: Color,
    pub info: NodeInfo,
    pub body: Vec<ParseNode>,
}
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct ColorTokenNode {
    pub color: Color,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct OrdGroupNode {
    pub body: Vec<ParseNode>,
    pub semi_simple: Option<bool>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct RawNode {
    pub string: String,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct SizeNode {
    pub value: Measurement,
    pub is_blank: bool,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct StylingNode {
    pub style: Style,
    pub body: Vec<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct SupSubNode {
    pub base: Option<Box<ParseNode>>,
    pub sup: Option<Box<ParseNode>>,
    pub sub: Option<Box<ParseNode>>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct TagNode {
    pub body: Vec<ParseNode>,
    pub tag: Vec<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct TextNode {
    pub body: Vec<ParseNode>,
    pub font: Option<Cow<'static, str>>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct UrlNode {
    pub url: String,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct VerbNode {
    pub body: Cow<'static, str>,
    pub star: bool,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct AtomNode {
    pub family: Atom,
    pub text: String,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct MathOrdNode {
    pub text: String,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct SpacingNode {
    pub text: String,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct TextOrdNode {
    pub text: Cow<'static, str>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct AccentTokenNode {
    pub text: String,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct OpTokenNode {
    pub text: String,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct AccentNode {
    pub label: Cow<'static, str>,
    pub is_stretchy: Option<bool>,
    pub is_shifty: Option<bool>,
    pub base: Box<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct AccentUnderNode {
    pub label: String,
    pub is_stretchy: Option<bool>,
    pub is_shifty: Option<bool>,
    pub base: Box<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct CrNode {
    pub new_line: bool,
    pub size: Option<Measurement>,
    pub info: NodeInfo,
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

#[derive(Debug, Clone)]
pub struct DelimSizingNode {
    pub size: DelimSize,
    pub m_class: MClass,
    pub delim: Cow<'static, str>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct EncloseNode {
    pub label: String,
    // TODO: Should this be a more general color
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct EnvironmentNode {
    pub name: String,
    pub name_group: Box<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct FontNode {
    pub font: Cow<'static, str>,
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct HBoxNode {
    pub body: Vec<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct HorizBraceNode {
    pub label: String,
    pub is_over: bool,
    pub base: Box<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct HrefNode {
    pub href: String,
    pub body: Vec<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct HtmlNode {
    pub attributes: HashMap<String, String>,
    pub body: Vec<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct HtmlMathMLNode {
    pub html: Vec<ParseNode>,
    pub math_ml: Vec<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct IncludeGraphicsNode {
    pub alt: String,
    pub width: Measurement,
    pub height: Measurement,
    pub total_height: Measurement,
    pub src: String,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct InfixNode {
    pub replace_with: Cow<'static, str>,
    pub size: Option<Measurement>,
    pub token: Option<Token<'static>>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct InternalNode {
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct KernNode {
    pub dimension: Measurement,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct LapNode {
    pub alignment: String,
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct LeftRightNode {
    pub alignment: String,
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct LeftRightRightNode {
    pub delim: String,
    pub color: Option<Color>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct MathChoiceNode {
    pub display: Vec<ParseNode>,
    pub text: Vec<ParseNode>,
    pub script: Vec<ParseNode>,
    pub script_script: Vec<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct MiddleNode {
    pub delim: String,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct MClassNode {
    pub m_class: String,
    pub body: Vec<ParseNode>,
    pub is_character_box: bool,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct OperatorNameNode {
    pub body: Vec<ParseNode>,
    pub always_handle_sup_sub: bool,
    pub limits: bool,
    pub parent_is_sup_sub: bool,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct OverlineNode {
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct PhantomNode {
    pub body: Vec<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct HPhantomNode {
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct VPhantomNode {
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct RaiseBoxNode {
    pub dy: Measurement,
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct RuleNode {
    pub shift: Option<Measurement>,
    pub width: Measurement,
    pub height: Measurement,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct SizingNode {
    // TODO: floating point?
    pub size: usize,
    pub body: Vec<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct SmashNode {
    pub body: Box<ParseNode>,
    pub smash_height: bool,
    pub smash_depth: bool,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct SqrtNode {
    pub body: Box<ParseNode>,
    pub index: Option<Box<ParseNode>>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct UnderlineNode {
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct VCenterNode {
    pub body: Box<ParseNode>,
    pub info: NodeInfo,
}

#[derive(Debug, Clone)]
pub struct XArrowNode {
    pub label: String,
    pub body: Box<ParseNode>,
    pub below: Option<Box<ParseNode>>,
    pub info: NodeInfo,
}
