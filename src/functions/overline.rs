use std::borrow::Cow;
use std::sync::Arc;

use crate::build_common::{self, make_span, VListElem, VListKern, VListShiftChild};
use crate::dom_tree::{CssStyle, HtmlNode};
use crate::html;
use crate::mathml;
use crate::mathml_tree::{MathNode, MathNodeType, MathmlNode, TextNode};
use crate::parse_node::{NodeInfo, OverlineNode, ParseNode, ParseNodeType};
use crate::tree::ClassList;

use super::{FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let overline = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Overline, 1),
        handler: Box::new(overline_handler),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });

    fns.insert(Cow::Borrowed("\\overline"), overline);
}

fn overline_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    ParseNode::Overline(OverlineNode {
        body: Box::new(args[0].clone()),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}

#[cfg(feature = "html")]
fn html_builder(group: &ParseNode, options: &crate::Options) -> HtmlNode {
    let ParseNode::Overline(group) = group else {
        panic!("Expected Overline node");
    };

    // Build the inner group in cramped style (TeXbook Rule 9).
    let cramped = options.having_cramped_style();
    let inner_options = cramped.as_ref().unwrap_or(options);
    let inner = html::build_group(Some(&group.body), inner_options, None);

    // Create the overline elements.
    let line = build_common::make_line_span("overline-line", options, None);
    let default_rule_thickness = options.font_metrics().default_rule_thickness;
    let vlist = build_common::make_v_list(
        build_common::VListParam::FirstBaseLine {
            children: vec![
                VListShiftChild::Elem(VListElem::new(inner)),
                VListShiftChild::Kern(VListKern(3.0 * default_rule_thickness)),
                VListShiftChild::Elem(VListElem::new(line.into())),
                VListShiftChild::Kern(VListKern(default_rule_thickness)),
            ],
        },
        options,
    );

    make_span::<HtmlNode>(
        vec!["mord".to_string(), "overline".to_string()],
        vec![vlist.into()],
        Some(options),
        CssStyle::default(),
    )
    .into()
}

#[cfg(feature = "mathml")]
fn mathml_builder(group: &ParseNode, options: &crate::Options) -> MathmlNode {
    let ParseNode::Overline(group) = group else {
        panic!("Expected Overline node");
    };

    let mut operator = MathNode::new(
        MathNodeType::Mo,
        vec![TextNode::new("\u{203e}".to_string())],
        ClassList::new(),
    );
    operator.set_attribute("stretchy", "true");

    let body = mathml::build_group(Some(&group.body), options);

    let mut node =
        MathNode::new(MathNodeType::MOver, vec![body, operator.into()], ClassList::new());
    node.set_attribute("accent", "true");

    node.into()
}
