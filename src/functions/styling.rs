use std::sync::Arc;

use crate::{
    mathml,
    mathml_tree::{MathNode, MathNodeType},
    parse_node::{NodeInfo, ParseNode, ParseNodeType, StylingNode},
    parser::ParseError,
    tree::ClassList,
    util::Style,
};

use super::{sizing::sizing_group, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let styling = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Styling, 0)
            .with_allowed_in_text(true)
            .with_primitive(true),
        handler: Box::new(|ctx, _, _| {
            // parse out the implicit body
            let body = ctx
                .parser
                .dispatch_parse_expression(true, ctx.break_on_token_text)
                .unwrap();

            let style = &ctx.func_name[1..ctx.func_name.len() - 5];
            let style = match style {
                "display" => Style::Display,
                "text" => Style::Text,
                "script" => Style::Script,
                "scriptscript" => Style::ScriptScript,
                _ => unreachable!(),
            };

            Ok(ParseNode::Styling(StylingNode {
                style,
                body,
                info: NodeInfo::new_mode(ctx.parser.mode()),
            }))
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            let ParseNode::Styling(group) = group else { unreachable!() };
            // Style changes are handled in the TeXbook on pg. 442, Rule 3.
            let new_style = group.style.into_style_id();
            let new_options = options.having_style(new_style);
            let new_options = new_options.as_ref().unwrap_or(options);
            let new_options = new_options.clone().with_font("");

            sizing_group(&group.body, &new_options, options).into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            let ParseNode::Styling(group) = group else { unreachable!() };

            let new_style = group.style.into_style_id();
            let new_options = options.having_style(new_style);
            let new_options = new_options.as_ref().unwrap_or(options);

            let inner = mathml::build_expression(&group.body, new_options, None);

            let mut node = MathNode::new(MathNodeType::MStyle, inner, ClassList::new());

            let (script_level, display_style) = match group.style {
                Style::Display => (0, "true"),
                Style::Text => (0, "false"),
                Style::Script => (1, "false"),
                Style::ScriptScript => (2, "false"),
            };

            node.set_attribute("scriptlevel", script_level.to_string());
            node.set_attribute("displaystyle", display_style.to_string());

            node.into()
        })),
    });

    fns.insert_for_all_str(
        [
            "\\displaystyle",
            "\\textstyle",
            "\\scriptstyle",
            "\\scriptscriptstyle",
        ]
        .into_iter(),
        styling,
    );
}
