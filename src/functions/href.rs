use std::{borrow::Cow, sync::Arc};

use crate::{
    build_common,
    html::{self, RealGroup},
    mathml,
    mathml_tree::{MathNode, MathNodeType, MathmlNode},
    parse_node::{HrefNode, NodeInfo, ParseNode, ParseNodeType, TextNode, TextOrdNode},
    tree::ClassList,
    util::ArgType,
};

use super::{ord_argument, FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let href = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Href, 2)
            .with_allowed_in_text(true)
            .with_arg_types(&[ArgType::Url, ArgType::Original] as &[ArgType]),
        handler: Box::new(|ctx: FunctionContext, args, _opt_args| {
            let body = args[1].clone();
            let ParseNode::Url(href) = &args[0] else {
                panic!()
            };
            let href = &href.url;

            // TODO: allow a function to be used
            if !ctx.parser.conf.is_trusted("\\href", href) {
                return ParseNode::Color(ctx.parser.format_unsupported_cmd("\\href"));
            }

            ParseNode::Href(HrefNode {
                href: href.clone(),
                body: ord_argument(body),
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            let ParseNode::Href(group) = group else {
                panic!()
            };

            let elements =
                html::build_expression(group.body.clone(), options, RealGroup::False, (None, None));

            build_common::make_anchor(group.href.clone(), ClassList::new(), elements, options)
                .into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            let ParseNode::Href(group) = group else {
                panic!()
            };

            let math = mathml::build_expression_row(group.body.clone(), options, None);
            let mut math = match math {
                MathmlNode::Math(math) => math,
                _ => MathNode::new(MathNodeType::MRow, vec![math], ClassList::new()),
            };
            math.set_attribute("href", &group.href);

            math.into()
        })),
    });

    fns.insert("\\href".into(), href);

    let url = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Url, 1)
            .with_allowed_in_text(true)
            .with_arg_types(&[ArgType::Url] as &[ArgType]),
        handler: Box::new(|ctx, args, _opt_args| {
            let ParseNode::Url(href) = &args[0] else {
                panic!()
            };
            let href = &href.url;

            if !ctx.parser.conf.is_trusted("\\url", href) {
                return ParseNode::Color(ctx.parser.format_unsupported_cmd("\\url"));
            }

            let mut chars = Vec::with_capacity(href.chars().count());
            for c in href.chars() {
                if c == '~' {
                    chars.push(ParseNode::TextOrd(TextOrdNode {
                        text: Cow::Borrowed("\\textasciitilde"),
                        info: NodeInfo::new_mode(ctx.parser.mode()),
                    }));
                } else {
                    chars.push(ParseNode::TextOrd(TextOrdNode {
                        text: Cow::Owned(c.to_string()),
                        info: NodeInfo::new_mode(ctx.parser.mode()),
                    }));
                }
            }

            let body = TextNode {
                body: chars,
                font: Some(Cow::Borrowed("\\texttt")),
                info: NodeInfo::new_mode(ctx.parser.mode()),
            };

            ParseNode::Href(HrefNode {
                href: href.clone(),
                body: ord_argument(ParseNode::Text(body)),
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });

    fns.insert("\\url".into(), url);
}
