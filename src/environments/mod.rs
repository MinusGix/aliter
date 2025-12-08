use std::{borrow::Cow, collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;

use crate::{expander::Mode, functions::FunctionPropSpec, parse_node::ParseNode, parser::Parser, parser::ParseError};

pub mod cd;
pub mod array;

pub struct EnvironmentContext<'a, 'p, 'i, 'f> {
    pub mode: Mode,
    pub env_name: Cow<'a, str>,
    pub parser: &'p mut Parser<'i, 'f>,
}

pub struct EnvironmentSpec {
    pub prop: FunctionPropSpec,
    pub handler:
        Box<dyn Fn(EnvironmentContext, &[ParseNode], &[Option<ParseNode>]) -> Result<ParseNode, ParseError> + Send + Sync>,
}

pub type Environments = HashMap<&'static str, Arc<EnvironmentSpec>>;

pub(crate) static ENVIRONMENTS: Lazy<Environments> = Lazy::new(|| {
    let mut envs = HashMap::new();

    array::add_environments(&mut envs);

    envs
});
