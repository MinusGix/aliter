use crate::{
    tree::{class_attr, Attributes, ClassList, VirtualNode},
    unit::{make_em, Em},
    util,
};

/// MathML node types used in Aliter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathNodeType {
    Math,
    Annotation,
    Semantics,
    MText,
    Mn,
    Mo,
    MSpace,
    MOver,
    MUnder,
    MUnderOver,
    MSup,
    MSub,
    MSubSup,
    MFrac,
    MRoot,
    MSqrt,
    MTable,
    MTr,
    MTd,
    MLabeledTr,
    MRow,
    MEnclose,
    MStyle,
    MPadded,
    MPhantom,
    MGlyph,
}
impl MathNodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MathNodeType::Math => "math",
            MathNodeType::Annotation => "annotation",
            MathNodeType::Semantics => "semantics",
            MathNodeType::MText => "mtext",
            MathNodeType::Mn => "mn",
            MathNodeType::Mo => "mo",
            MathNodeType::MSpace => "mspace",
            MathNodeType::MOver => "mover",
            MathNodeType::MUnder => "munder",
            MathNodeType::MUnderOver => "munderover",
            MathNodeType::MSup => "msup",
            MathNodeType::MSub => "msub",
            MathNodeType::MSubSup => "msubsup",
            MathNodeType::MFrac => "mfrac",
            MathNodeType::MRoot => "mroot",
            MathNodeType::MSqrt => "msqrt",
            MathNodeType::MTable => "mtable",
            MathNodeType::MTr => "mtr",
            MathNodeType::MTd => "mtd",
            MathNodeType::MLabeledTr => "mlabeledtr",
            MathNodeType::MRow => "mrow",
            MathNodeType::MEnclose => "menclose",
            MathNodeType::MStyle => "mstyle",
            MathNodeType::MPadded => "mpadded",
            MathNodeType::MPhantom => "mphantom",
            MathNodeType::MGlyph => "mglyph",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MathDomNode {}
impl VirtualNode for MathDomNode {
    fn to_markup(&self) -> String {
        todo!()
    }
}

/// A trait for nodes which contain an [`HtmlDomNode`]  
/// This is needed since some parts of KaTeX use [`HtmlDomNode`] like an abstract
/// base, but we're treating it like a normal structure
pub trait WithMathDomNode: VirtualNode {
    fn node(&self) -> &MathDomNode;

    fn node_mut(&mut self) -> &mut MathDomNode;
}
impl<T: WithMathDomNode + ?Sized> WithMathDomNode for Box<T> {
    fn node(&self) -> &MathDomNode {
        (**self).node()
    }

    fn node_mut(&mut self) -> &mut MathDomNode {
        (**self).node_mut()
    }
}
impl WithMathDomNode for MathDomNode {
    fn node(&self) -> &MathDomNode {
        self
    }

    fn node_mut(&mut self) -> &mut MathDomNode {
        self
    }
}

#[derive(Debug, Clone)]
pub struct EmptyMathNode {
    node: MathDomNode,
}
impl EmptyMathNode {
    pub fn new() -> EmptyMathNode {
        EmptyMathNode {
            node: MathDomNode {},
        }
    }
}
impl VirtualNode for EmptyMathNode {
    fn to_markup(&self) -> String {
        String::new()
    }
}
impl WithMathDomNode for EmptyMathNode {
    fn node(&self) -> &MathDomNode {
        &self.node
    }

    fn node_mut(&mut self) -> &mut MathDomNode {
        &mut self.node
    }
}

/// General purpose MathML node of any type.
#[derive(Debug, Clone)]
pub struct MathNode<T: WithMathDomNode> {
    pub node: MathDomNode,
    pub typ: MathNodeType,
    children: Vec<T>,
    pub classes: ClassList,
    attributes: Attributes,
}
impl<T: WithMathDomNode> MathNode<T> {
    pub fn new(typ: MathNodeType, children: Vec<T>, classes: ClassList) -> MathNode<T> {
        MathNode {
            node: MathDomNode {},
            typ,
            children,
            classes,
            attributes: Attributes::new(),
        }
    }

    pub fn new_empty(typ: MathNodeType) -> MathNode<T> {
        MathNode::new(typ, Vec::new(), ClassList::new())
    }

    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    pub fn get_attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(String::as_str)
    }

    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.set_attribute(key, value);
        self
    }

    // TODO: into_node
    // TODO: to_text
}
impl<T: WithMathDomNode> VirtualNode for MathNode<T> {
    fn to_markup(&self) -> String {
        let mut markup = format!("<{}", self.typ.as_str());

        // Add the attributes
        for (key, value) in self.attributes.iter() {
            markup.push(' ');
            markup.push_str(key);
            markup.push_str("=\"");

            let escaped_value = util::escape(value);
            markup.push_str(&escaped_value);
            markup.push('"');
        }

        if let Some(classes) = class_attr(&self.classes) {
            markup.push_str(" class=\"");
            markup.push_str(&classes);
            markup.push('"');
        }

        markup.push('>');

        for child in self.children.iter() {
            markup.push_str(&child.to_markup());
        }

        markup.push_str("</");
        markup.push_str(self.typ.as_str());
        markup.push('>');

        markup
    }
}
impl<T: WithMathDomNode> WithMathDomNode for MathNode<T> {
    fn node(&self) -> &MathDomNode {
        &self.node
    }

    fn node_mut(&mut self) -> &mut MathDomNode {
        &mut self.node
    }
}

/// Represents a piece of text
pub struct TextNode {
    pub node: MathDomNode,
    pub text: String,
}
impl TextNode {
    pub fn new(text: String) -> TextNode {
        TextNode {
            node: MathDomNode {},
            text,
        }
    }

    // TODO: should this be in a trait?
    pub fn to_text(&self) -> String {
        self.text.clone()
    }
}
impl VirtualNode for TextNode {
    fn to_markup(&self) -> String {
        util::escape(&self.text).into()
    }
}
impl WithMathDomNode for TextNode {
    fn node(&self) -> &MathDomNode {
        &self.node
    }

    fn node_mut(&mut self) -> &mut MathDomNode {
        &mut self.node
    }
}

/// Represents a space, but may render as `<mspace.../>` or as text, depending on the width
pub struct SpaceNode {
    pub node: MathDomNode,
    pub width: Em,
    pub character: Option<(char, Option<char>)>,
}
impl SpaceNode {
    pub fn new(width: Em) -> SpaceNode {
        let character = Self::choose_space_like_character(width);

        SpaceNode {
            node: MathDomNode {},
            width,
            character,
        }
    }

    fn choose_space_like_character(width: Em) -> Option<(char, Option<char>)> {
        let width = width.0;
        // See https://www.w3.org/TR/2000/WD-MathML2-20000328/chapter6.html
        // for a table of space-like characters.  We use Unicode
        // representations instead of &LongNames; as it's not clear how to
        // make the latter via document.createTextNode.
        Some(if width >= 0.05555 && width <= 0.05556 {
            // &VeryThinSpace;
            ('\u{200a}', None)
        } else if width >= 0.1666 && width <= 0.1667 {
            // &ThinSpace;
            ('\u{2009}', None)
        } else if width >= 0.2222 && width <= 0.2223 {
            // &MediumSpace;
            ('\u{205f}', None)
        } else if width >= 0.2777 && width <= 0.2778 {
            // &ThickSpace;
            ('\u{2005}', Some('\u{200a}'))
        } else if width >= -0.05556 && width <= -0.05555 {
            // &NegativeVeryThinSpace;
            ('\u{200a}', Some('\u{2063}'))
        } else if width >= -0.1667 && width <= -0.1666 {
            // &NegativeThinSpace;
            ('\u{2009}', Some('\u{2063}'))
        } else if width >= -0.2223 && width <= -0.2222 {
            // &NegativeMediumSpace;
            ('\u{205f}', Some('\u{2063}'))
        } else if width >= -0.2778 && width <= -0.2777 {
            // &NegativeThickSpace;
            ('\u{2005}', Some('\u{2063}'))
        } else {
            return None;
        })
    }

    // TODO: to node

    pub fn to_text(&self) -> String {
        if let Some((ch1, ch2)) = self.character {
            if let Some(ch2) = ch2 {
                format!("{}{}", ch1, ch2)
            } else {
                ch1.to_string()
            }
        } else {
            " ".to_string()
        }
    }
}
impl VirtualNode for SpaceNode {
    fn to_markup(&self) -> String {
        if let Some((ch1, ch2)) = self.character {
            if let Some(ch2) = ch2 {
                format!("<mtext>{}{}</mtext>", ch1, ch2)
            } else {
                format!("<mtext>{}</mtext>", ch1)
            }
        } else {
            let width = make_em(self.width.0);
            format!("<mspace width=\"{}\"/>", width)
        }
    }
}
impl WithMathDomNode for SpaceNode {
    fn node(&self) -> &MathDomNode {
        &self.node
    }

    fn node_mut(&mut self) -> &mut MathDomNode {
        &mut self.node
    }
}
