use crate::dom_tree::HtmlDomNode;

// TODO: Vec of enum for common kinds?
pub type ClassList = Vec<String>;

pub trait VirtualNode {
    // TODO: We somehow need to support translating into HTML nodes
    // We could use websys?
    // fn into_node(self) -> Node;
    fn to_markup(&self) -> String;
}
impl<T: VirtualNode + ?Sized> VirtualNode for Box<T> {
    fn to_markup(&self) -> String {
        (**self).to_markup()
    }
}

// TODO: implements htmldomnode, mathdomnode..
pub struct DocumentFragment<T: VirtualNode> {
    pub node: HtmlDomNode,
    pub children: Vec<T>,
}
impl<T: VirtualNode> DocumentFragment<T> {
    pub fn new(children: Vec<T>) -> DocumentFragment<T> {
        DocumentFragment {
            node: HtmlDomNode::default(),
            children,
        }
    }

    pub fn has_class(&self, class: &str) -> bool {
        self.node.has_class(class)
    }

    pub fn to_markup(&self) -> String {
        self.children.iter().map(|c| c.to_markup()).collect()
    }

    // TODO: math node to text?
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct EmptyNode;
impl VirtualNode for EmptyNode {
    fn to_markup(&self) -> String {
        String::new()
    }
}
