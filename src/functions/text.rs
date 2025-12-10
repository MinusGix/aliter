use std::{borrow::Cow, sync::Arc};

use crate::{
    build_common::make_span,
    dom_tree::CssStyle,
    expander::Mode,
    html, mathml,
    parse_node::{NodeInfo, ParseNode, ParseNodeType, TextNode},
    parser::ParseError,
    util::ArgType,
    FontShape, FontWeight, Options,
};

use super::{ord_argument, FunctionPropSpec, FunctionSpec, Functions};

fn options_with_font(group: &TextNode, options: Options) -> Options {
    if let Some(font) = group.font.as_deref() {
        match font {
            "\\textrm" => options.with_text_font_family("textrm"),
            "\\textsf" => options.with_text_font_family("textsf"),
            "\\texttt" => options.with_text_font_family("texttt"),
            "\\textnormal" => options.with_text_font_family("textnormal"),
            // TODO: katex includes \\text but sets it to undefind which.. doesn't seem like it does anything?
            "\\textbf" => options.with_text_font_weight(FontWeight::TextBf),
            "\\textmd" => options.with_text_font_weight(FontWeight::TextMd),

            "\\textit" => options.with_text_font_shape(FontShape::TextIt),
            "\\textup" => options.with_text_font_shape(FontShape::TextUp),

            // fallback: leave options unchanged
            _ => options,
        }
    } else {
        options
    }
}

pub fn add_functions(fns: &mut Functions) {
    let text = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Text, 1)
            .with_allowed_in_argument(true)
            .with_allowed_in_text(true)
            .with_arg_types(&[ArgType::Mode(Mode::Text)] as &[ArgType]),
        handler: Box::new(|ctx, args, _opt_args| {
            let body = args[0].clone();
            Ok(ParseNode::Text(TextNode {
                body: ord_argument(body),
                font: Some(Cow::Owned(ctx.func_name.to_string())),
                info: NodeInfo::new_mode(ctx.parser.mode()),
            }))
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            let ParseNode::Text(group) = group else { unreachable!() };
            let options = options_with_font(group, options.clone_alter());
            let inner =
                html::build_expression(&group.body, &options, html::RealGroup::True, (None, None));

            make_span(
                vec!["mord".to_string(), "text".to_string()],
                inner,
                Some(&options),
                CssStyle::default(),
            )
            .into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            let ParseNode::Text(group) = group else { unreachable!() };
            let options = options_with_font(group, options.clone_alter());
            mathml::build_expression_row(&group.body, &options, None)
        })),
    });

    fns.insert_for_all_str(
        [
            // Font families
            "\\text",
            "\\textrm",
            "\\textsf",
            "\\texttt",
            "\\textnormal",
            // Font weights
            "\\textbf",
            "\\textmd",
            // Font Shapes
            "\\textit",
            "\\textup",
        ]
        .into_iter(),
        text,
    );
}
