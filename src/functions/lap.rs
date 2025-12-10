use std::{borrow::Cow, sync::Arc};

use crate::{
    build_common::{make_span, make_span_s},
    dom_tree::CssStyle,
    html, mathml,
    mathml_tree::{MathNode, MathNodeType},
    parse_node::{LapNode, NodeInfo, ParseNode, ParseNodeType},
    parser::ParseError,
    tree::ClassList,
    unit::make_em,
};

use super::{FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let lap = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Lap, 1).with_allowed_in_text(true),
        handler: Box::new(|ctx, args, _opt_args| {
            let body = args[0].clone();
            Ok(ParseNode::Lap(LapNode {
                alignment: ctx.func_name[5..].to_string(),
                body: Box::new(body),
                info: NodeInfo::new_mode(ctx.parser.mode()),
            }))
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            let ParseNode::Lap(group) = group else {
                panic!();
            };

            let inner = html::build_group(Some(&group.body), options, None);
            let inner = make_span_s(ClassList::new(), vec![inner]);
            let inner = if group.alignment == "clap" {
                // ref: https://www.math.lsu.edu/~aperlis/publications/mathclap/
                // wrap, since css will center a .clap > .inner > span

                make_span(
                    vec!["inner".to_string()],
                    vec![inner],
                    Some(options),
                    CssStyle::default(),
                )
                .using_html_node()
            } else {
                inner
            };

            let fix = make_span_s(vec!["fix".to_string()], Vec::new());
            let mut node = make_span(
                vec![group.alignment.clone()],
                vec![inner, fix],
                Some(options),
                CssStyle::default(),
            );

            // At this point, we have correctly set horizontal alignment of the
            // two items involved in the lap.
            // Next, use a strut to set the height of the HTML bounding box.
            // Otherwise, a tall argument may be misplaced.
            // This code resolved issue #1153
            let mut strut = make_span_s(vec!["span".to_string()], Vec::new());
            strut.node.style.height = Some(Cow::Owned(make_em(node.node.height + node.node.depth)));
            // TODO: katex includes it if it is defined. Do they allow it to be undefined?
            // if node.node.depth != 0.0 {
            strut.node.style.vertical_align = Some(Cow::Owned(make_em(-node.node.depth)));
            // }

            node.children.insert(0, strut);

            // Next, prevent vertical misplacement when next to something tall.
            // This code resolved issue #1234
            let node = make_span(
                vec!["thinbox".to_string()],
                vec![node],
                Some(options),
                CssStyle::default(),
            );

            make_span(
                vec!["mord".to_string(), "vbox".to_string()],
                vec![node],
                Some(options),
                CssStyle::default(),
            )
            .into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            let ParseNode::Lap(group) = group else {
                panic!();
            };
            let node = mathml::build_group(Some(&group.body), options);
            let mut node = MathNode::new(MathNodeType::MPadded, vec![node], ClassList::new());

            if group.alignment != "rlap" {
                let offset = if group.alignment == "llap" {
                    "-1width"
                } else {
                    "-0.5width"
                };
                node.set_attribute("lspace", offset);
            }
            node.set_attribute("width", "0px");

            node.into()
        })),
    });

    fns.insert_for_all_str(["\\mathllap", "\\mathrlap", "\\mathclap"].into_iter(), lap);
}
