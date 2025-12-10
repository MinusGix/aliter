use std::sync::Arc;

use crate::{
    build_common::make_fragment,
    dom_tree::{DocumentFragment, HtmlNode},
    html::{self, RealGroup},
    mathml,
    mathml_tree::{MathNode, MathNodeType},
    parse_node::{NodeInfo, ParseNode, ParseNodeType, SizingNode},
    parser::ParseError,
    tree::ClassList,
    unit::make_em,
    Options,
};

use super::{FunctionPropSpec, FunctionSpec, Functions};

const SIZE_FUNCS: &'static [&'static str] = &[
    "\\tiny",
    "\\sixptsize",
    "\\scriptsize",
    "\\footnotesize",
    "\\small",
    "\\normalsize",
    "\\large",
    "\\Large",
    "\\LARGE",
    "\\huge",
    "\\Huge",
];

pub fn add_functions(fns: &mut Functions) {
    let sizing = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Sizing, 0).with_allowed_in_text(true),
        handler: Box::new(|ctx, _, _| {
            let body = ctx
                .parser
                .dispatch_parse_expression(false, ctx.break_on_token_text)
                .unwrap();

            Ok(ParseNode::Sizing(SizingNode {
                size: SIZE_FUNCS.iter().position(|&s| s == ctx.func_name).unwrap() + 1,
                body,
                info: NodeInfo::new_mode(ctx.parser.mode()),
            }))
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            let ParseNode::Sizing(group) = group else { unreachable!() };

            let new_options = options.having_size(group.size);
            let new_options = new_options.as_ref().unwrap_or(options);

            let inner = mathml::build_expression(&group.body, new_options, None);

            let mut node = MathNode::new(MathNodeType::MStyle, inner, ClassList::new());

            // TODO: This doesn't produce the correct size for nested size
            // changes, because we don't keep state of what style we're currently
            // in, so we can't reset the size to normal before changing it.  Now
            // that we're passing an options parameter we should be able to fix
            // this.
            node.set_attribute("mathsize", make_em(new_options.size_multiplier()));

            node.into()
        })),
    });

    fns.insert_for_all_str(SIZE_FUNCS.iter().copied(), sizing);
}

// #[cfg(feature = "html")]
pub(crate) fn html_builder(group: &ParseNode, options: &Options) -> HtmlNode {
    // Handle sizing operators like \Huge. Real TeX doesn't actually allow
    // these functions inside of math expressions, so we do some special
    // handling.
    let ParseNode::Sizing(group) = group else { unreachable!() };
    let new_options = options.having_size(group.size);
    let new_options = new_options.as_ref().unwrap_or(options);

    sizing_group(&group.body, new_options, options).into()
}

pub(crate) fn sizing_group(
    value: &[ParseNode],
    options: &Options,
    base_options: &Options,
) -> DocumentFragment<HtmlNode> {
    use crate::dom_tree::WithHtmlDomNode;

    let mut inner = html::build_expression(value, options, RealGroup::False, (None, None));
    let mult = options.size_multiplier() / base_options.size_multiplier();

    // TODO: We should be able to do this without formatting
    let reset_size = format!("reset-size{}", options.size);
    // Add size-resetting classes to the innter list and set max_font_size manually.
    // Handle nested size changes.
    for node in inner.iter_mut() {
        let pos = node.node().classes.iter().position(|c| c == "sizing");
        if let Some(pos) = pos {
            if node.node().classes.get(pos + 1) == Some(&reset_size) {
                // This is a nested size change: e.g., node is the "b" in
                // `\Huge a \small b`. Override the old size (the `reset-` class)
                // but not the new size.
                node.node_mut().classes[pos + 1] = format!("reset-size{}", base_options.size);
            }
        } else {
            node.node_mut()
                .classes
                .extend(options.sizing_classes(base_options));
        }

        node.node_mut().height *= mult;
        node.node_mut().depth *= mult;
    }

    make_fragment(inner)
}
