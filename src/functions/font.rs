use std::{borrow::Cow, sync::Arc};

use crate::{
    html,
    parse_node::{FontNode, MClassNode, NodeInfo, OrdGroupNode, ParseNode, ParseNodeType},
    parser::ParseError,
    util, Options,
};

#[cfg(feature = "html")]
use crate::dom_tree::HtmlNode;
#[cfg(feature = "mathml")]
use crate::mathml_tree::MathmlNode;

use super::{mclass::bin_rel_class, normalize_argument, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let math = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Font, 1).with_allowed_in_argument(true),
        handler: Box::new(|ctx, args, _opt_args| {
            let body = normalize_argument(args[0].clone());
            let func = match ctx.func_name.as_ref() {
                "\\Bbb" => Cow::Borrowed("mathbb"),
                "\\bold" => Cow::Borrowed("mathbf"),
                "\\frak" => Cow::Borrowed("mathfrak"),
                "\\bm" => Cow::Borrowed("boldsymbol"),
                x => Cow::Owned(x[1..].to_string()),
            };

            Ok(ParseNode::Font(FontNode {
                font: func,
                body: Box::new(body),
                info: NodeInfo::new_mode(ctx.parser.mode()),
            }))
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });

    fns.insert_for_all_str(
        [
            // styles, except \boldsymbol defined below
            "\\mathrm",
            "\\mathit",
            "\\mathbf",
            "\\mathnormal",
            // families
            "\\mathbb",
            "\\mathcal",
            "\\mathfrak",
            "\\mathscr",
            "\\mathsf",
            "\\mathtt",
            // aliases, except \bm defined below
            "\\Bbb",
            "\\bold",
            "\\frak",
        ]
        .into_iter(),
        math,
    );

    let mclass = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::MClass, 1),
        handler: Box::new(|ctx, args, _opt_args| {
            let body = args[0].clone();
            let is_character_box = util::is_character_box(&body);
            // amsbsy.sty's \boldsymbol uses \binrel spacing to inherit the
            // argument's bin|rel|ord status
            Ok(ParseNode::MClass(MClassNode {
                m_class: bin_rel_class(&body),
                body: vec![ParseNode::Font(FontNode {
                    font: Cow::Borrowed("boldsymbol"),
                    body: Box::new(body),
                    info: NodeInfo::new_mode(ctx.parser.mode()),
                })],
                is_character_box,
                info: NodeInfo::new_mode(ctx.parser.mode()),
            }))
        }),
        #[cfg(feature = "html")]
        html_builder: None,
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });

    fns.insert_for_all_str(["\\boldsymbol", "\\bm"].into_iter(), mclass);

    // Old font changing functions
    let font = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Font, 0).with_allowed_in_text(true),
        handler: Box::new(|ctx, _args, _opt_args| {
            let body = ctx
                .parser
                .dispatch_parse_expression(true, ctx.break_on_token_text)
                .unwrap();
            let style = format!("math{}", &ctx.func_name[1..]);

            Ok(ParseNode::Font(FontNode {
                font: Cow::Owned(style),
                body: Box::new(ParseNode::OrdGroup(OrdGroupNode {
                    body,
                    semi_simple: None,
                    info: NodeInfo::new_mode(ctx.parser.mode()),
                })),
                info: NodeInfo::new_mode(ctx.parser.mode()),
            }))
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });

    fns.insert_for_all_str(
        ["\\rm", "\\sf", "\\tt", "\\bf", "\\it", "\\cal"].into_iter(),
        font,
    );
}

#[cfg(feature = "html")]
fn html_builder(group: &ParseNode, options: &Options) -> HtmlNode {
    let ParseNode::Font(group) = group else {
        panic!();
    };

    let new_options = options.clone().with_font(group.font.clone());

    html::build_group(Some(&group.body), &new_options, None)
}

#[cfg(feature = "mathml")]
fn mathml_builder(group: &ParseNode, options: &Options) -> MathmlNode {
    use crate::mathml;

    let ParseNode::Font(group) = group else {
        panic!();
    };

    let new_options = options.clone().with_font(group.font.clone());

    mathml::build_group(Some(&group.body), &new_options)
}
