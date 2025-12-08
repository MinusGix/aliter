use std::sync::Arc;

use crate::{
    build_common::{self, make_span, VListElem, VListShiftChild},
    delimiter,
    dom_tree::{CssStyle, HtmlNode, WithHtmlDomNode},
    html, mathml,
    parse_node::{NodeInfo, ParseNode, ParseNodeType, SqrtNode},
    style,
    unit::make_em,
};

use super::{FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

#[cfg(feature = "mathml")]
use crate::mathml_tree::{MathNode, MathNodeType, MathmlNode};
#[cfg(feature = "mathml")]
use crate::tree::ClassList;

pub fn add_functions(fns: &mut Functions) {
    let sqrt = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_opt_args(ParseNodeType::Sqrt, 1, 1)
            .with_allowed_in_argument(true),
        handler: Box::new(sqrt_handler),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });
    fns.insert("\\sqrt".into(), sqrt);
}

fn sqrt_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    opt_args: &[Option<ParseNode>],
) -> ParseNode {
    let index = opt_args[0].clone().map(Box::new);
    let body = Box::new(args[0].clone());
    ParseNode::Sqrt(SqrtNode {
        body,
        index,
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}

#[cfg(feature = "html")]
fn html_builder(group: &ParseNode, options: &crate::Options) -> HtmlNode {
    let ParseNode::Sqrt(group) = group else {
        panic!("Expected Sqrt node");
    };

    // Square roots are handled in the TeXbook pg. 443, Rule 11.

    // First, we do the same steps as in overline to build the inner group
    // and line
    let new_options = options.having_cramped_style();
    let inner_options = new_options.as_ref().unwrap_or(options);
    let mut inner = html::build_group(Some(&group.body), inner_options, None);
    if inner.node().height == 0.0 {
        // Render a small surd.
        inner.node_mut().height = options.font_metrics().x_height;
    }

    // Some groups can return document fragments.  Handle those by wrapping
    // them in a span.
    let mut inner = if let HtmlNode::DocumentFragment(_) = inner {
        make_span::<HtmlNode>(vec![], vec![inner], Some(options), CssStyle::default())
    } else {
        if let HtmlNode::Span(s) = inner {
            s
        } else {
             make_span::<HtmlNode>(vec![], vec![inner], Some(options), CssStyle::default())
        }
    };

    // Calculate the minimum size for the \surd delimiter
    let metrics = options.font_metrics();
    let theta = metrics.default_rule_thickness;

    let mut phi = theta;
    if options.style < style::TEXT_STYLE {
        phi = options.font_metrics().x_height;
    }

    // Calculate the clearance between the body and line
    let mut line_clearance = theta + phi / 4.0;

    let min_delimiter_height = inner.node().height + inner.node().depth + line_clearance + theta;

    // Create a sqrt SVG of the required minimum size
    let img_info = delimiter::make_sqrt_image(min_delimiter_height, options);
    let img = img_info.span;
    let rule_width = img_info.rule_width;
    let advance_width = img_info.advance_width;

    let delim_depth = img.node.height - rule_width;

    // Adjust the clearance based on the delimiter size
    if delim_depth > inner.node().height + inner.node().depth + line_clearance {
        line_clearance =
            (line_clearance + delim_depth - inner.node().height - inner.node().depth) / 2.0;
    }

    // Shift the sqrt image
    let img_shift = img.node.height - inner.node.height - line_clearance - rule_width;

    inner.node.style.padding_left = Some(make_em(advance_width).into());

    let inner_height = inner.node().height;

    // Overlay the image and the argument.
    let body = build_common::make_v_list(
        build_common::VListParam::FirstBaseLine {
            children: vec![
                VListShiftChild::Elem(VListElem {
                    elem: inner.using_html_node(),
                    wrapper_classes: vec!["svg-align".to_string()].into(),
                    margin_left: None,
                    margin_right: None,
                    wrapper_style: CssStyle::default(),
                }),
                VListShiftChild::Kern(build_common::VListKern(-(inner_height + img_shift))),
                VListShiftChild::Elem(VListElem {
                    elem: img.using_html_node(),
                    wrapper_classes: crate::tree::ClassList::new(),
                    margin_left: None,
                    margin_right: None,
                    wrapper_style: CssStyle::default(),
                }),
                VListShiftChild::Kern(build_common::VListKern(rule_width)),
            ],
        },
        options,
    );

    if group.index.is_none() {
        return make_span::<HtmlNode>(
            vec!["mord".to_string(), "sqrt".to_string()],
            vec![body.into()],
            Some(options),
            CssStyle::default(),
        )
        .into();
    } else {
        // Handle the optional root index
        let index = group.index.as_ref().unwrap();

        // The index is always in scriptscript style
        let new_options = options.having_style(style::SCRIPT_SCRIPT_STYLE);
        let root_options = new_options.as_ref().unwrap_or(options);
        let rootm = html::build_group(Some(index), root_options, Some(options));

        // The amount the index is shifted by. This is taken from the TeX
        // source, in the definition of `\r@@t`.
        let to_shift = 0.6 * (body.node().height - body.node().depth);

        // Build a VList with the superscript shifted up correctly
        let root_vlist = build_common::make_v_list(
            build_common::VListParam::Shift {
                amount: -to_shift,
                children: vec![VListShiftChild::Elem(VListElem {
                    elem: match rootm {
                        HtmlNode::Span(s) => s.using_html_node(),
                        _ => make_span::<HtmlNode>(vec![], vec![rootm], Some(root_options), CssStyle::default())
                            .using_html_node(),
                    },
                    wrapper_classes: crate::tree::ClassList::new(),
                    margin_left: None,
                    margin_right: None,
                    wrapper_style: CssStyle::default(),
                })],
            },
            options,
        );

        // Add a class surrounding it so we can add on the appropriate
        // kerning
        let root_vlist_wrap = make_span::<HtmlNode>(
            vec!["root".to_string()],
            vec![root_vlist.into()],
            None,
            CssStyle::default(),
        );

        return make_span::<HtmlNode>(
            vec!["mord".to_string(), "sqrt".to_string()],
            vec![root_vlist_wrap.into(), body.into()],
            Some(options),
            CssStyle::default(),
        )
        .into();
    }
}

#[cfg(feature = "mathml")]
fn mathml_builder(group: &ParseNode, options: &crate::Options) -> MathmlNode {
    let ParseNode::Sqrt(node) = group else {
        panic!("Expected Sqrt node");
    };
    let body = mathml::build_group(Some(&node.body), options);
    if let Some(index) = &node.index {
        let index = mathml::build_group(Some(index), options);
        MathNode::new(MathNodeType::MRoot, vec![body, index], ClassList::new()).into()
    } else {
        MathNode::new(MathNodeType::MSqrt, vec![body], ClassList::new()).into()
    }
}