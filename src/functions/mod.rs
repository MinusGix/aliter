use std::{borrow::Cow, collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;

use crate::{
    environments::cd,
    expander::BreakToken,
    lexer::Token,
    parse_node::{ParseNode, ParseNodeType},
    parser::Parser,
    util::ArgType,
    Options,
};

#[cfg(feature = "html")]
use crate::dom_tree::HtmlNode;
#[cfg(feature = "mathml")]
use crate::mathml_tree::MathmlNode;

pub mod accent;
pub mod accent_under;
pub mod arrow;
pub mod char;
pub mod color;
pub mod cr;
pub mod def;
pub mod genfrac;

// TODO: Put specific function groups under features? Eh
pub(crate) const FUNCTIONS: Lazy<Functions> = Lazy::new(|| {
    let mut fns = Functions {
        fns: HashMap::new(),
    };

    accent::add_functions(&mut fns);
    accent_under::add_functions(&mut fns);
    arrow::add_functions(&mut fns);
    cd::add_functions(&mut fns);
    crate::functions::char::add_functions(&mut fns);
    color::add_functions(&mut fns);
    cr::add_functions(&mut fns);
    def::add_functions(&mut fns);
    genfrac::add_functions(&mut fns);

    fns
});

#[derive(Clone)]
pub struct Functions {
    fns: HashMap<Cow<'static, str>, Arc<FunctionSpec>>,
}
impl Functions {
    pub fn get(&self, name: &str) -> Option<&Arc<FunctionSpec>> {
        self.fns.get(name)
    }

    pub fn insert(&mut self, name: Cow<'static, str>, spec: Arc<FunctionSpec>) {
        self.fns.insert(name.into(), spec);
    }

    pub fn insert_for_all_str<I: Iterator<Item = &'static str>>(
        &mut self,
        names: I,
        spec: Arc<FunctionSpec>,
    ) {
        for name in names {
            self.insert(Cow::Borrowed(name), spec.clone())
        }
    }

    pub fn find_html_builder_for_type(&self, typ: ParseNodeType) -> Option<&HtmlBuilderFn> {
        for spec in self.fns.values() {
            if spec.prop.typ == typ {
                return spec.html_builder.as_ref();
            }
        }

        None
    }

    pub fn find_mathml_builder_for_type(&self, typ: ParseNodeType) -> Option<&MathmlBuilderFn> {
        for spec in self.fns.values() {
            if spec.prop.typ == typ {
                return spec.mathml_builder.as_ref();
            }
        }

        None
    }
}

pub struct FunctionContext<'a, 'p, 'i, 'f> {
    pub func_name: Cow<'a, str>,
    pub parser: &'p mut Parser<'i, 'f>,
    pub token: Option<Token<'i>>,
    pub break_on_token_text: Option<BreakToken>,
}

#[derive(Debug, Clone)]
pub struct FunctionPropSpec {
    pub typ: ParseNodeType,
    /// Number of arguments the function takes
    pub num_args: usize,
    /// Array for each argument in the function, giving the type
    /// of the argument that should be parsed.  
    /// Its length should equal `num_optional_args + num_args`
    /// and types for optional arguments should appear before types for mandatory arguments
    pub arg_types: Cow<'static, [ArgType]>,
    /// Whether it expands to a single token or a braced group of tokens.  
    /// If it's grouped, it can be used as an argument to primitive commands,
    /// such as \sqrt (without the optional argument) and super/subscript.
    pub allowed_in_argument: bool,
    /// Whether or not the function is allowed inside text mode
    pub allowed_in_text: bool,
    /// Whether or not the function is allowed inside math mode
    pub allowed_in_math: bool,
    /// The number of optional arguments the function should parse
    pub num_optional_args: usize,
    /// Must be true if the function is an infix operator
    pub infix: bool,
    pub primitive: bool,
}
impl FunctionPropSpec {
    /// Create function prop spec with only num args set
    pub const fn new_num_args(typ: ParseNodeType, num_args: usize) -> FunctionPropSpec {
        Self::new_num_opt_args(typ, num_args, 0)
    }

    /// Create function prop spec with num args and opt args set
    pub const fn new_num_opt_args(
        typ: ParseNodeType,
        num_args: usize,
        opt_args: usize,
    ) -> FunctionPropSpec {
        FunctionPropSpec {
            typ,
            num_args,
            arg_types: Cow::Borrowed(&[]),
            allowed_in_argument: false,
            allowed_in_text: false,
            allowed_in_math: true,
            num_optional_args: opt_args,
            infix: false,
            primitive: false,
        }
    }

    pub(crate) fn with_allowed_in_text(mut self, allowed_in_text: bool) -> Self {
        self.allowed_in_text = allowed_in_text;
        self
    }

    pub(crate) fn with_allowed_in_math(mut self, allowed_in_math: bool) -> Self {
        self.allowed_in_math = allowed_in_math;
        self
    }

    pub(crate) fn with_allowed_in_argument(mut self, allowed_in_argument: bool) -> Self {
        self.allowed_in_argument = allowed_in_argument;
        self
    }

    pub(crate) fn with_primitive(mut self, primitive: bool) -> Self {
        self.primitive = primitive;
        self
    }

    pub(crate) fn with_arg_types(mut self, arg_types: impl Into<Cow<'static, [ArgType]>>) -> Self {
        self.arg_types = arg_types.into();
        self
    }

    pub(crate) fn with_infix(mut self, infix: bool) -> Self {
        self.infix = infix;
        self
    }
}

#[cfg(feature = "html")]
pub type HtmlBuilderFn = Box<dyn Fn(&ParseNode, &Options) -> HtmlNode>;
#[cfg(feature = "mathml")]
pub type MathmlBuilderFn = Box<dyn Fn(&ParseNode, &Options) -> MathmlNode>;

pub struct FunctionSpec {
    pub prop: FunctionPropSpec,
    /// (context, args, opt_args)
    pub handler: Box<dyn Fn(FunctionContext, &[ParseNode], &[Option<ParseNode>]) -> ParseNode>,
    #[cfg(feature = "html")]
    pub html_builder: Option<HtmlBuilderFn>,
    #[cfg(feature = "mathml")]
    pub mathml_builder: Option<MathmlBuilderFn>,
}

pub fn normalize_argument(arg: ParseNode) -> ParseNode {
    if let ParseNode::OrdGroup(node) = arg {
        if node.body.len() == 1 {
            node.body.into_iter().next().unwrap()
        } else {
            ParseNode::OrdGroup(node)
        }
    } else {
        arg
    }
}

/// If the argument is an ord group, we just return the body
/// Otherwise it is just a vec of the given arg
pub(crate) fn ord_argument(arg: ParseNode) -> Vec<ParseNode> {
    if let ParseNode::OrdGroup(ord_group) = arg {
        ord_group.body
    } else {
        vec![arg]
    }
}
