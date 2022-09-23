use std::{borrow::Cow, sync::Arc};

use crate::{
    functions::{FunctionContext, FunctionPropSpec, FunctionSpec, Functions},
    parse_node::{CdLabelNode, CdLabelParentNode, NodeInfo, ParseNode, ParseNodeType},
};

pub fn add_functions(fns: &mut Functions) {
    let cd = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::CdLabel, 1),
        handler: Box::new(cd_handler),
    });

    fns.insert_for_all_str(CD_NAMES.iter().copied(), cd);

    let cd_label = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::CdLabelParentNode, 1),
        handler: Box::new(cd_label_parent_handler),
    });

    fns.insert(Cow::Borrowed("\\\\cdparent"), cd_label);
}

const CD_NAMES: &'static [&'static str] = &["\\\\cdleft", "\\\\cdright"];

fn cd_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    ParseNode::CdLabel(CdLabelNode {
        side: (&ctx.func_name[4..]).to_string().into(),
        label: Box::new(args[0].clone()),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}

fn cd_label_parent_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    ParseNode::CdLabelParentNode(CdLabelParentNode {
        fragment: Box::new(args[0].clone()),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}
