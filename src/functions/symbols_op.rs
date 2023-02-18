use std::sync::Arc;

use crate::{
    build_common::math_sym,
    mathml,
    mathml_tree::{MathNode, MathNodeType},
    parse_node::{ParseNode, ParseNodeType},
    symbols::Atom,
    tree::ClassList,
    util::FontVariant,
};

use super::{BuilderFunctionSpec, FunctionPropSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let atom = Arc::new(BuilderFunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Atom, 0),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            let ParseNode::Atom(atom) = group else {
                panic!();
            };

            let class = format!("m{}", atom.family.as_str());

            math_sym(&atom.text, atom.info.mode, options, vec![class]).into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            let ParseNode::Atom(atom) = group else {
                panic!();
            };

            let text = mathml::make_text(atom.text.to_string(), atom.info.mode, Some(options));

            let mut node = MathNode::new(MathNodeType::Mo, vec![text], ClassList::new());

            match atom.family {
                Atom::Bin => {
                    let variant = mathml::get_variant(atom, options);
                    if variant == Some(FontVariant::BoldItalic) {
                        node.set_attribute("mathvariant", "bold-italic");
                    }
                }
                Atom::Punct => {
                    node.set_attribute("separator", "true");
                }
                Atom::Open | Atom::Close => {
                    // Delims built here should not stretch vertically.
                    node.set_attribute("stretchy", "false");
                }
                _ => {}
            }

            node.into()
        })),
    });

    fns.insert_builder(atom);
}
