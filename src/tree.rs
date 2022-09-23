use crate::dom_tree::HtmlDomNode;

pub trait VirtualNode {
    // TODO: We somehow need to support translating into HTML nodes
    // We could use websys?
    // fn into_node(self) -> Node;
    fn to_markup(&self) -> String;
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
