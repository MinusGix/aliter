use crate::tree::{Attributes, ClassList, VirtualNode};

/// MathML node types used in Aliter.
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

pub struct MathNode<T: WithMathDomNode> {
    pub node: MathDomNode,
    pub typ: MathNodeType,
    children: Vec<T>,
    pub classes: ClassList,
    pub attributes: Attributes,
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
}
