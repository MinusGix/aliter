use crate::{
    functions,
    mathml_tree::{EmptyMathNode, MathNode, MathNodeType, WithMathDomNode},
    parse_node::ParseNode,
    Options,
};

pub(crate) fn build_group(
    group: Option<&ParseNode>,
    options: &Options,
) -> Box<dyn WithMathDomNode> {
    let Some(group) = group else {
        return Box::new(MathNode::<EmptyMathNode>::new_empty(MathNodeType::MRow));
    };

    if let Some(mathml_builder) = functions::FUNCTIONS.find_mathml_builder_for_type(group.typ()) {
        mathml_builder(group, options)
    } else {
        panic!("Got group of unknown type")
    }
}
