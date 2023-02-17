use std::{borrow::Cow, collections::HashMap};

use crate::util;

// TODO: Vec of enum for common kinds?
pub type ClassList = Vec<String>;

// TODO: We could do better by having keys be Cow<'static, str>?
// Though I think you need a crate for a nicely behaving map type for that
pub type Attributes = HashMap<String, String>;

/// Returns the value that should go in `class="{}"`
pub(crate) fn class_attr(classes: &ClassList) -> Option<String> {
    if classes.is_empty() {
        None
    } else {
        // TODO: use intersperse instead
        Some(
            classes
                .iter()
                .map(|class| util::escape(class.as_str()))
                .collect::<Vec<Cow<'_, str>>>()
                .join(" "),
        )
    }
}

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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct EmptyNode;
impl VirtualNode for EmptyNode {
    fn to_markup(&self) -> String {
        String::new()
    }
}
