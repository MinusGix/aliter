use std::sync::Arc;

use crate::macr::{MacroReplace, Macros};

#[derive(Debug, Clone)]
pub struct Namespace {
    pub(crate) current: Macros,
    builtins: Macros,
    undefined_stack: Vec<Macros<Option<Arc<MacroReplace>>>>,
}
impl Namespace {
    pub fn new(builtins: Macros, global_macros: Macros) -> Namespace {
        Self {
            current: global_macros,
            builtins,
            undefined_stack: Vec::new(),
        }
    }

    /// Start a new nested group
    pub fn begin_group(&mut self) {
        self.undefined_stack.push(Macros::default());
    }

    /// End current nested group, restore values from before the group began.
    pub fn end_group(&mut self) {
        debug_assert!(self.undefined_stack.is_empty(), "Unbalanced namespace destruction: attempted to pop global namespace. This is a library bug, please report it.");

        if let Some(undefs) = self.undefined_stack.pop() {
            let (back, letter) = undefs.into_macros_iters();
            let back = back.filter_map(|(n, v)| v.map(|v| (n, v)));
            let letter = letter.filter_map(|(n, v)| v.map(|v| (n, v)));
            self.current.insert_macros_iter(back, letter);
        }
        // otherwise, ignore
    }

    pub fn end_groups(&mut self) {
        while !self.undefined_stack.is_empty() {
            self.end_group();
        }
    }

    /// Check whether the macro exists
    pub fn contains_back_macro(&self, name: &str) -> bool {
        self.current.contains_back_macro(name) || self.builtins.contains_back_macro(name)
    }

    pub fn get_back_macro(&self, name: &str) -> Option<&Arc<MacroReplace>> {
        self.current.get_back_macro(name)
    }

    pub fn set_global_back_macro(&mut self, name: String, repl: Option<Arc<MacroReplace>>) {
        // Global set is equivalent to setting in all groups.
        // We can simulate that by removing it from the undefined stack (so that other versions of
        // it don't get undone and remove this version)
        for undef in &mut self.undefined_stack {
            let _ = undef.take_back_macro(&name);
        }

        if let Some(repl) = &repl {
            self.current.insert_back_macro(name.clone(), repl.clone());
        } else {
            let _ = self.current.take_back_macro(&name);
        }

        if let Some(undef) = self.undefined_stack.last_mut() {
            undef.insert_back_macro(name, repl);
        }
    }

    pub fn set_back_macro(&mut self, name: String, repl: Option<Arc<MacroReplace>>) {
        // Undo this set at the end of this group, unless an undo already is already in place
        // in which case, that older value is the correct one.

        if let Some(undef) = self.undefined_stack.last_mut() {
            if !undef.contains_back_macro(&name) {
                undef.insert_back_macro(name.clone(), self.current.get_back_macro(&name).cloned());
            }
        }

        if let Some(repl) = repl {
            self.current.insert_back_macro(name, repl);
        } else {
            let _ = self.current.take_back_macro(&name);
        }
    }
}
