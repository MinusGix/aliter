use std::borrow::Cow;

use crate::{
    mathml_tree::{MathDomNode, MathmlNode, WithMathDomNode},
    parse_node::Color,
    svg_geometry,
    tree::{class_attr, Attributes, ClassList, EmptyNode, VirtualNode},
    util::{self, find_assoc_data},
    Options,
};

// TODO: We could have so fields that are numbers have to be actual numbers?
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CssStyle {
    pub background_color: Option<Cow<'static, str>>,
    pub border_bottom_width: Option<Cow<'static, str>>,
    pub border_color: Option<Cow<'static, str>>,
    pub border_right_style: Option<Cow<'static, str>>,
    pub border_right_width: Option<Cow<'static, str>>,
    pub border_top_width: Option<Cow<'static, str>>,
    pub border_style: Option<Cow<'static, str>>,
    pub border_width: Option<Cow<'static, str>>,
    pub bottom: Option<Cow<'static, str>>,
    pub color: Option<Color>,
    pub height: Option<Cow<'static, str>>,
    pub left: Option<Cow<'static, str>>,
    pub margin: Option<Cow<'static, str>>,
    pub margin_left: Option<Cow<'static, str>>,
    pub margin_right: Option<Cow<'static, str>>,
    pub margin_top: Option<Cow<'static, str>>,
    pub min_width: Option<Cow<'static, str>>,
    pub padding_left: Option<Cow<'static, str>>,
    pub position: Option<Cow<'static, str>>,
    pub top: Option<Cow<'static, str>>,
    pub width: Option<Cow<'static, str>>,
    pub vertical_align: Option<Cow<'static, str>>,
}
impl CssStyle {
    // Technically could be modified to return just a `Cow<'static, str>` if there was only one set
    /// Convert this into how it would be if in the `style=""` attribute  
    pub fn as_style_attr(&self) -> Option<String> {
        let mut res = String::new();

        let mut append_field = |field_name: &'static str, val: &Option<Cow<'static, str>>| {
            if let Some(val) = val {
                res.push_str(field_name);
                res.push_str(": ");
                res.push_str(val);
                res.push(';');
            }
        };

        append_field("background-color", &self.background_color);
        append_field("border-bottom-width", &self.border_bottom_width);
        append_field("border-color", &self.border_color);
        append_field("border-right-style", &self.border_right_style);
        append_field("border-right-width", &self.border_right_width);
        append_field("border-top-width", &self.border_top_width);
        append_field("border-style", &self.border_style);
        append_field("border-width", &self.border_width);
        append_field("bottom", &self.bottom);
        append_field("height", &self.height);
        append_field("left", &self.left);
        append_field("margin", &self.margin);
        append_field("margin-left", &self.margin_left);
        append_field("margin-right", &self.margin_right);
        append_field("margin-top", &self.margin_top);
        append_field("min-width", &self.min_width);
        append_field("padding-left", &self.padding_left);
        append_field("position", &self.position);
        append_field("top", &self.top);
        append_field("width", &self.width);
        append_field("vertical-align", &self.vertical_align);

        if let Some(color) = &self.color {
            let color = color.to_string();
            res.push_str("color: ");
            res.push_str(&color);
            res.push(';');
        }

        if res.is_empty() {
            None
        } else {
            Some(res)
        }
    }
}

fn to_markup_attr<T: VirtualNode>(
    node: &HtmlDomNode,
    attributes: &Attributes,
    children: &[T],
    tag_name: &str,
) -> String {
    let mut markup = format!("<{tag_name}");

    if let Some(classes) = class_attr(&node.classes) {
        markup.push_str(" class=\"");
        markup.push_str(&classes);
        markup.push('"');
    }

    if let Some(style) = node.style.as_style_attr() {
        markup.push_str(" style=\"");
        markup.push_str(&style);
        markup.push('"');
    }

    for (name, val) in attributes.iter() {
        markup.push(' ');
        markup.push_str(name);
        markup.push_str("=\"");
        markup.push_str(&util::escape(val));
        markup.push('"');
    }

    markup.push('>');

    for child in children {
        markup.push_str(&child.to_markup());
    }

    markup.push_str("</");
    markup.push_str(tag_name);
    markup.push('>');

    markup
}

// Note: Unlike KaTeX, we have this as a shared structure for use as a field
// rather than an interface. This is primarily because Rust does not have
// field-interfaces, and while we could use a trait that is not a nice
// implementation. So, instead we have a structure that another structure
// can just include.
#[derive(Debug, Clone, PartialEq)]
pub struct HtmlDomNode {
    pub classes: ClassList,
    pub height: f64,
    pub depth: f64,
    pub max_font_size: f64,
    pub style: CssStyle,
}
impl HtmlDomNode {
    pub fn new(
        mut classes: ClassList,
        options: Option<&Options>,
        mut style: CssStyle,
    ) -> HtmlDomNode {
        if let Some(options) = options {
            if options.style.is_tight() {
                classes.push("mtight".to_owned());
            }

            if let Some(color) = options.get_color() {
                style.color = Some(color);
            }
        }

        HtmlDomNode {
            classes,
            height: 0.0,
            depth: 0.0,
            max_font_size: 0.0,
            style,
        }
    }

    pub fn has_class(&self, class: &str) -> bool {
        self.classes.iter().any(|x| x == class)
    }
}
impl Default for HtmlDomNode {
    fn default() -> Self {
        Self {
            classes: ClassList::new(),
            height: 0.0,
            depth: 0.0,
            max_font_size: 0.0,
            style: CssStyle::default(),
        }
    }
}
impl VirtualNode for HtmlDomNode {
    fn to_markup(&self) -> String {
        // Bare HtmlDomNode has no tag context; emit an empty span to expose classes/style.
        let mut attrs = Attributes::new();
        if let Some(style) = self.style.as_style_attr() {
            attrs.insert("style".to_string(), style);
        }
        let mut dummy = Span {
            node: self.clone(),
            children: Vec::<EmptyNode>::new(),
            attributes: attrs,
            width: None,
        };
        dummy.to_markup()
    }
}

#[derive(Debug, Clone)]
pub enum HtmlNode {
    Empty(EmptyNode),
    DocumentFragment(DocumentFragment<HtmlNode>),
    Span(Span<HtmlNode>),
    Anchor(Anchor<HtmlNode>),
    Img(Img),
    Symbol(SymbolNode),
    Svg(SvgNode),
}
impl VirtualNode for HtmlNode {
    fn to_markup(&self) -> String {
        match self {
            HtmlNode::Empty(node) => node.to_markup(),
            HtmlNode::DocumentFragment(node) => node.to_markup(),
            HtmlNode::Span(node) => node.to_markup(),
            HtmlNode::Anchor(node) => node.to_markup(),
            HtmlNode::Img(node) => node.to_markup(),
            HtmlNode::Symbol(node) => node.to_markup(),
            HtmlNode::Svg(node) => node.to_markup(),
        }
    }
}
impl WithHtmlDomNode for HtmlNode {
    fn node(&self) -> &HtmlDomNode {
        match self {
            HtmlNode::Empty(node) => node.node(),
            HtmlNode::DocumentFragment(node) => WithHtmlDomNode::node(node),
            HtmlNode::Span(node) => node.node(),
            HtmlNode::Anchor(node) => node.node(),
            HtmlNode::Img(node) => node.node(),
            HtmlNode::Symbol(node) => node.node(),
            HtmlNode::Svg(node) => node.node(),
        }
    }

    fn node_mut(&mut self) -> &mut HtmlDomNode {
        match self {
            HtmlNode::Empty(node) => node.node_mut(),
            HtmlNode::DocumentFragment(node) => WithHtmlDomNode::node_mut(node),
            HtmlNode::Span(node) => node.node_mut(),
            HtmlNode::Anchor(node) => node.node_mut(),
            HtmlNode::Img(node) => node.node_mut(),
            HtmlNode::Symbol(node) => node.node_mut(),
            HtmlNode::Svg(node) => node.node_mut(),
        }
    }
}
impl From<EmptyNode> for HtmlNode {
    fn from(node: EmptyNode) -> Self {
        HtmlNode::Empty(node)
    }
}
impl<T: VirtualNode> From<DocumentFragment<T>> for HtmlNode
where
    HtmlNode: From<T>,
{
    fn from(node: DocumentFragment<T>) -> Self {
        let node = node.using_html_node();
        HtmlNode::DocumentFragment(node)
    }
}
impl<T: VirtualNode> From<Span<T>> for HtmlNode
where
    HtmlNode: From<T>,
{
    fn from(node: Span<T>) -> Self {
        let node = node.using_html_node();
        HtmlNode::Span(node)
    }
}
impl From<Anchor<HtmlNode>> for HtmlNode {
    fn from(node: Anchor<HtmlNode>) -> Self {
        HtmlNode::Anchor(node)
    }
}
impl From<Img> for HtmlNode {
    fn from(node: Img) -> Self {
        HtmlNode::Img(node)
    }
}
impl From<SymbolNode> for HtmlNode {
    fn from(node: SymbolNode) -> Self {
        HtmlNode::Symbol(node)
    }
}
impl From<SvgNode> for HtmlNode {
    fn from(node: SvgNode) -> Self {
        HtmlNode::Svg(node)
    }
}

/// A trait for nodes which contain an [`HtmlDomNode`]  
/// This is needed since some parts of KaTeX use [`HtmlDomNode`] like an abstract
/// base, but we're treating it like a normal structure
pub trait WithHtmlDomNode: VirtualNode {
    fn node(&self) -> &HtmlDomNode;

    fn node_mut(&mut self) -> &mut HtmlDomNode;
}
impl<T: WithHtmlDomNode + ?Sized> WithHtmlDomNode for Box<T> {
    fn node(&self) -> &HtmlDomNode {
        (**self).node()
    }

    fn node_mut(&mut self) -> &mut HtmlDomNode {
        (**self).node_mut()
    }
}
impl WithHtmlDomNode for HtmlDomNode {
    fn node(&self) -> &HtmlDomNode {
        self
    }

    fn node_mut(&mut self) -> &mut HtmlDomNode {
        self
    }
}

// TODO: implements htmldomnode, mathdomnode..
#[derive(Debug, Clone)]
pub struct DocumentFragment<T: VirtualNode> {
    #[cfg(feature = "html")]
    pub node: HtmlDomNode,
    #[cfg(feature = "mathml")]
    pub math_node: MathDomNode,
    pub children: Vec<T>,
}
impl<T: VirtualNode> DocumentFragment<T> {
    pub fn new(children: Vec<T>) -> DocumentFragment<T> {
        DocumentFragment {
            #[cfg(feature = "html")]
            node: HtmlDomNode::default(),
            #[cfg(feature = "mathml")]
            math_node: MathDomNode {},
            children,
        }
    }

    #[cfg(feature = "html")]
    pub fn has_class(&self, class: &str) -> bool {
        self.node.has_class(class)
    }

    // TODO: math node to text?
}
#[cfg(feature = "html")]
impl<T: VirtualNode> DocumentFragment<T>
where
    HtmlNode: From<T>,
{
    pub fn using_html_node(self) -> DocumentFragment<HtmlNode> {
        DocumentFragment {
            node: self.node,
            #[cfg(feature = "mathml")]
            math_node: self.math_node,
            children: self.children.into_iter().map(HtmlNode::from).collect(),
        }
    }
}
#[cfg(feature = "mathml")]
impl<T: VirtualNode> DocumentFragment<T>
where
    MathmlNode: From<T>,
{
    pub fn using_mathml_node(self) -> DocumentFragment<MathmlNode> {
        DocumentFragment {
            node: self.node,
            math_node: self.math_node,
            children: self.children.into_iter().map(MathmlNode::from).collect(),
        }
    }
}
impl<T: VirtualNode> VirtualNode for DocumentFragment<T> {
    fn to_markup(&self) -> String {
        self.children.iter().map(|c| c.to_markup()).collect()
    }
}
#[cfg(feature = "html")]
impl<T: VirtualNode> WithHtmlDomNode for DocumentFragment<T> {
    fn node(&self) -> &HtmlDomNode {
        &self.node
    }

    fn node_mut(&mut self) -> &mut HtmlDomNode {
        &mut self.node
    }
}
#[cfg(feature = "mathml")]
impl<T: VirtualNode> WithMathDomNode for DocumentFragment<T> {
    fn node(&self) -> &MathDomNode {
        &self.math_node
    }

    fn node_mut(&mut self) -> &mut MathDomNode {
        &mut self.math_node
    }
}

pub type DomSpan = Span<Box<dyn WithHtmlDomNode>>;
// TODO: pub type SvgSpan
// TODO: pub type SvgChildNode
pub type DocumentFragmentNode = DocumentFragment<Box<dyn WithHtmlDomNode>>;

#[derive(Debug, Clone)]
pub struct Span<T: VirtualNode> {
    pub node: HtmlDomNode,
    pub children: Vec<T>,
    pub attributes: Attributes,
    pub width: Option<f64>,
}
impl<T: VirtualNode> Span<T> {
    pub fn new(
        classes: ClassList,
        children: Vec<T>,
        options: Option<&Options>,
        style: CssStyle,
    ) -> Span<T> {
        let node = HtmlDomNode::new(classes, options, style);
        Span {
            node,
            children,
            attributes: Attributes::new(),
            width: None,
        }
    }

    pub fn map<U: VirtualNode>(self, f: impl Fn(T) -> U) -> Span<U> {
        Span {
            node: self.node,
            children: self.children.into_iter().map(f).collect(),
            attributes: self.attributes,
            width: self.width,
        }
    }
}
impl<T: VirtualNode> Span<T>
where
    HtmlNode: From<T>,
{
    pub fn using_html_node(self) -> Span<HtmlNode> {
        self.map(HtmlNode::from)
    }
}
impl<T: WithHtmlDomNode + 'static> Span<T> {
    pub fn into_dom_span(self) -> DomSpan {
        Span {
            node: self.node,
            children: self
                .children
                .into_iter()
                .map(|child| Box::new(child) as Box<dyn WithHtmlDomNode>)
                .collect(),
            attributes: self.attributes,
            width: self.width,
        }
    }
}
impl<T: VirtualNode> Default for Span<T> {
    fn default() -> Self {
        Span {
            node: HtmlDomNode::new(ClassList::new(), None, CssStyle::default()),
            children: Vec::new(),
            attributes: Attributes::new(),
            width: None,
        }
    }
}
impl<T: VirtualNode> WithHtmlDomNode for Span<T> {
    fn node(&self) -> &HtmlDomNode {
        &self.node
    }

    fn node_mut(&mut self) -> &mut HtmlDomNode {
        &mut self.node
    }
}
impl<T: VirtualNode> VirtualNode for Span<T> {
    fn to_markup(&self) -> String {
        to_markup_attr(&self.node, &self.attributes, &self.children, "span")
    }
}

#[derive(Debug, Clone)]
pub struct Anchor<T: VirtualNode> {
    pub node: HtmlDomNode,
    pub children: Vec<T>,
    pub attributes: Attributes,
}
impl<T: VirtualNode> Anchor<T> {
    pub fn new(href: String, classes: ClassList, children: Vec<T>, options: &Options) -> Anchor<T> {
        let node = HtmlDomNode::new(classes, Some(options), CssStyle::default());
        let mut attributes = Attributes::new();
        attributes.insert("href".to_string(), href);

        Anchor {
            node,
            children,
            attributes,
        }
    }
}
impl<T: VirtualNode> WithHtmlDomNode for Anchor<T> {
    fn node(&self) -> &HtmlDomNode {
        &self.node
    }

    fn node_mut(&mut self) -> &mut HtmlDomNode {
        &mut self.node
    }
}
impl<T: VirtualNode> VirtualNode for Anchor<T> {
    fn to_markup(&self) -> String {
        to_markup_attr(&self.node, &self.attributes, &self.children, "a")
    }
}

#[derive(Debug, Clone)]
pub struct Img {
    pub node: HtmlDomNode,
    pub src: String,
    pub alt: String,
}
impl Img {
    pub fn new(src: String, alt: String, style: CssStyle) -> Img {
        let node = HtmlDomNode::new(vec!["mord".to_string()], None, style);
        Img { node, src, alt }
    }
}
impl WithHtmlDomNode for Img {
    fn node(&self) -> &HtmlDomNode {
        &self.node
    }

    fn node_mut(&mut self) -> &mut HtmlDomNode {
        &mut self.node
    }
}
impl VirtualNode for Img {
    fn to_markup(&self) -> String {
        // Note: This ignoring classes is what the KaTeX code already does
        let mut markup = format!("<img src=\"{}\" alt=\"{}\"", self.src, self.alt);

        if let Some(style) = self.node.style.as_style_attr() {
            markup.push_str(" style=\"");
            markup.push_str(&util::escape(&style));
            markup.push('"');
        }

        markup.push_str("'/>");

        markup
    }
}

/// A symbol node contains information about a single symbol.  
/// It either renders to a single text node, or a span with a single text node in it,
/// depending on whether has CSS classes, styles, or needs italic correction.
#[derive(Debug, Clone)]
pub struct SymbolNode {
    pub node: HtmlDomNode,
    pub text: String,
    pub italic: f64,
    pub skew: f64,
    pub width: f64,
}
impl SymbolNode {
    pub fn new(
        text: String,
        height: Option<f64>,
        depth: Option<f64>,
        italic: Option<f64>,
        skew: Option<f64>,
        width: Option<f64>,
        classes: ClassList,
        style: CssStyle,
    ) -> SymbolNode {
        let mut node = HtmlDomNode::new(classes, None, style);

        if let Some(height) = height {
            node.height = height;
        }

        if let Some(depth) = depth {
            node.depth = depth;
        }

        // FIXME: We currently ignore script!
        // FIXME: We currently ignore the special i + diacritic character handling

        SymbolNode {
            node,
            text,
            italic: italic.unwrap_or(0.0),
            skew: skew.unwrap_or(0.0),
            width: width.unwrap_or(0.0),
        }
    }

    pub fn new_text(text: String) -> SymbolNode {
        Self::new(
            text,
            None,
            None,
            None,
            None,
            None,
            ClassList::new(),
            CssStyle::default(),
        )
    }

    pub fn new_text_classes(text: String, classes: ClassList) -> SymbolNode {
        Self::new(
            text,
            None,
            None,
            None,
            None,
            None,
            classes,
            CssStyle::default(),
        )
    }
}
impl WithHtmlDomNode for SymbolNode {
    fn node(&self) -> &HtmlDomNode {
        &self.node
    }

    fn node_mut(&mut self) -> &mut HtmlDomNode {
        &mut self.node
    }
}
impl VirtualNode for SymbolNode {
    fn to_markup(&self) -> String {
        let mut needs_span = false;

        let mut markup = "<span".to_string();

        if let Some(classes) = class_attr(&self.node.classes) {
            needs_span = true;
            markup.push_str(" class=\"");
            markup.push_str(&classes);
            markup.push('"');
        }

        let italic_style = if self.italic > 0.0 {
            Some(format!("margin-right:{}em;", self.italic))
        } else {
            None
        };

        let styles = self.node.style.as_style_attr();

        if styles.is_some() || italic_style.is_some() {
            needs_span = true;
            markup.push_str(" style=\"");

            if let Some(italic_style) = italic_style {
                markup.push_str(&italic_style);
            }

            if let Some(styles) = styles {
                markup.push_str(&styles);
            }

            markup.push('"');
        }

        let escaped = util::escape(&self.text);
        if needs_span {
            markup.push('>');
            markup.push_str(&escaped);
            markup.push_str("</span>");

            markup
        } else {
            escaped.into_owned()
        }
    }
}

/// SVG nodes are used to render stretchy wide elements.
#[derive(Debug, Clone)]
pub struct SvgNode {
    pub node: HtmlDomNode,
    pub children: Vec<SvgChildNode>,
    pub attributes: Attributes,
}
impl SvgNode {
    pub fn new(children: Vec<SvgChildNode>) -> SvgNode {
        SvgNode {
            // Note: this ignores classes and styles
            node: HtmlDomNode::new(ClassList::new(), None, CssStyle::default()),
            children,
            attributes: Attributes::new(),
        }
    }

    pub fn with_attribute(mut self, key: impl Into<String>, val: impl Into<String>) -> SvgNode {
        self.attributes.insert(key.into(), val.into());
        self
    }

    // TODO: to_node
}
impl VirtualNode for SvgNode {
    fn to_markup(&self) -> String {
        let mut markup = "<svg xmlns=\"http://www.w3.org/2000/svg\"".to_string();

        // Apply attributes
        for (key, val) in self.attributes.iter() {
            markup.push(' ');
            markup.push_str(key);
            markup.push_str("=\'");
            markup.push_str(val);
            markup.push('\'');
        }

        markup.push('>');

        for child in &self.children {
            markup.push_str(&child.to_markup());
        }

        markup.push_str("</svg>");

        markup
    }
}
impl WithHtmlDomNode for SvgNode {
    fn node(&self) -> &HtmlDomNode {
        &self.node
    }

    fn node_mut(&mut self) -> &mut HtmlDomNode {
        &mut self.node
    }
}

#[derive(Debug, Clone)]
pub enum SvgChildNode {
    Path(PathNode),
    Line(LineNode),
}
impl SvgChildNode {
    pub fn to_markup(&self) -> String {
        match self {
            SvgChildNode::Path(path) => path.to_markup(),
            SvgChildNode::Line(line) => line.to_markup(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PathNode {
    pub path_name: Cow<'static, str>,
    pub alternate: Option<String>,
}
impl PathNode {
    pub fn new(path_name: impl Into<Cow<'static, str>>, alternate: Option<String>) -> PathNode {
        PathNode {
            path_name: path_name.into(),
            alternate,
        }
    }

    // TODO: to_node
}
impl VirtualNode for PathNode {
    fn to_markup(&self) -> String {
        if let Some(alternate) = self.alternate.as_deref() {
            format!("<path d='{}'/>", alternate)
        } else {
            let path = find_assoc_data(svg_geometry::PATH, &self.path_name).unwrap();
            format!("<path d='{}'/>", path)
        }
    }
}

#[derive(Debug, Clone)]
pub struct LineNode {
    pub attributes: Attributes,
}
impl LineNode {
    pub fn new(attributes: Attributes) -> LineNode {
        LineNode { attributes }
    }
}
impl VirtualNode for LineNode {
    fn to_markup(&self) -> String {
        let mut markup = "<line".to_string();

        // Apply attributes
        for (key, val) in self.attributes.iter() {
            markup.push(' ');
            markup.push_str(key);
            markup.push_str("=\'");
            markup.push_str(val);
            markup.push('\'');
        }

        markup.push_str("/>");

        markup
    }
}
