use std::borrow::Cow;
use std::sync::Arc;

use crate::build_common::{self, make_span, VListElem, VListKern, VListShiftChild};
use crate::dom_tree::{CssStyle, HtmlNode, WithHtmlDomNode};
use crate::html;
use crate::mathml;
use crate::mathml_tree::{MathNode, MathNodeType, MathmlNode, TextNode};
use crate::parse_node::{NodeInfo, ParseNode, ParseNodeType, UnderlineNode};
use crate::tree::ClassList;

use super::{FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let underline = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Underline, 1)
            .with_allowed_in_text(true),
        handler: Box::new(underline_handler),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });

    fns.insert(Cow::Borrowed("\\underline"), underline);
}

fn underline_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    ParseNode::Underline(UnderlineNode {
        body: Box::new(args[0].clone()),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}

#[cfg(feature = "html")]
fn html_builder(group: &ParseNode, options: &crate::Options) -> HtmlNode {
    let ParseNode::Underline(group) = group else {
        panic!("Expected Underline node");
    };

    // Underlines follow TeXbook Rule 10.
    let inner = html::build_group(Some(&group.body), options, None);
    let line = build_common::make_line_span("underline-line", options, None);
    let default_rule_thickness = options.font_metrics().default_rule_thickness;

    let vlist = build_common::make_v_list(
        build_common::VListParam::Top {
            amount: inner.node().height,
            children: vec![
                VListShiftChild::Kern(VListKern(default_rule_thickness)),
                VListShiftChild::Elem(VListElem::new(line.into())),
                VListShiftChild::Kern(VListKern(3.0 * default_rule_thickness)),
                VListShiftChild::Elem(VListElem::new(inner)),
            ],
        },
        options,
    );

    make_span::<HtmlNode>(
        vec!["mord".to_string(), "underline".to_string()],
        vec![vlist.into()],
        Some(options),
        CssStyle::default(),
    )
    .into()
}

#[cfg(feature = "mathml")]
fn mathml_builder(group: &ParseNode, options: &crate::Options) -> MathmlNode {
    let ParseNode::Underline(group) = group else {
        panic!("Expected Underline node");
    };

    let mut operator = MathNode::new(
        MathNodeType::Mo,
        vec![TextNode::new("\u{203e}".to_string())],
        ClassList::new(),
    );
    operator.set_attribute("stretchy", "true");

    let body = mathml::build_group(Some(&group.body), options);

    let mut node =
        MathNode::new(MathNodeType::MUnder, vec![body, operator.into()], ClassList::new());
    node.set_attribute("accentunder", "true");

    node.into()
}
