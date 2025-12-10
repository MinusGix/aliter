use std::sync::Arc;

use crate::{
    parse_node::{NodeInfo, ParseNode, ParseNodeType, RuleNode},
    util::ArgType,
};

use super::{FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let rule = Arc::new(FunctionSpec {
        // 2 required args + 1 optional arg, all must be sizes
        prop: FunctionPropSpec::new_num_opt_args(ParseNodeType::Rule, 2, 1)
            .with_allowed_in_text(true)
            // All three args (including optional shift) must be sizes
            .with_arg_types(&[ArgType::Size, ArgType::Size, ArgType::Size] as &[ArgType]),
        handler: Box::new(|ctx, args, opt_args| {
            // Optional: shift (raise/lower)
            let shift = if let Some(ParseNode::Size(size)) = opt_args.first().cloned().flatten() {
                Some(size.value)
            } else {
                None
            };

            // Required: width
            let ParseNode::Size(width) = args[0].clone() else {
                panic!("Expected size for rule width")
            };

            // Required: height
            let ParseNode::Size(height) = args[1].clone() else {
                panic!("Expected size for rule height")
            };

            ParseNode::Rule(RuleNode {
                shift,
                width: width.value,
                height: height.value,
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(|group, options| {
            use std::borrow::Cow;
            use crate::{build_common::make_span, dom_tree::{CssStyle, HtmlNode}, unit::{calculate_size, make_em}};

            let ParseNode::Rule(group) = group else { unreachable!() };

            // Make an empty span for the rule
            let mut rule = make_span::<HtmlNode>(
                vec!["mord".into(), "rule".into()],
                Vec::new(),
                Some(options),
                CssStyle::default(),
            );

            // Calculate the shift, width, and height of the rule, and account for units
            let width = calculate_size(&group.width, options);
            let height = calculate_size(&group.height, options);
            let shift = group.shift.as_ref().map(|s| calculate_size(s, options)).unwrap_or(0.0);

            // Style the rule to the right size
            rule.node.style.border_right_width = Some(Cow::Owned(make_em(width)));
            rule.node.style.border_top_width = Some(Cow::Owned(make_em(height)));
            rule.node.style.bottom = Some(Cow::Owned(make_em(shift)));

            // Record the height and width
            rule.node.height = height + shift;
            rule.node.depth = -shift;
            // Font size is the number large enough that the browser will
            // reserve at least `absHeight` space above the baseline.
            // The 1.125 factor was empirically determined
            rule.node.max_font_size = height * 1.125 * options.size_multiplier();

            rule.into()
        })),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(|group, options| {
            use crate::{mathml_tree::{MathNode, MathNodeType, MathmlNode}, tree::ClassList, unit::{calculate_size, make_em}};

            let ParseNode::Rule(group) = group else { unreachable!() };

            let width = calculate_size(&group.width, options);
            let height = calculate_size(&group.height, options);
            let shift = group.shift.as_ref().map(|s| calculate_size(s, options)).unwrap_or(0.0);
            let color = options.get_color().map(|c| c.to_string()).unwrap_or_else(|| "black".to_string());

            let mut rule: MathNode<MathmlNode> = MathNode::new_empty(MathNodeType::MSpace);
            rule.set_attribute("mathbackground", &color);
            rule.set_attribute("width", &make_em(width));
            rule.set_attribute("height", &make_em(height));

            let mut wrapper: MathNode<MathmlNode> = MathNode::new(MathNodeType::MPadded, vec![rule.into()], ClassList::new());
            if shift >= 0.0 {
                wrapper.set_attribute("height", &make_em(shift));
            } else {
                wrapper.set_attribute("height", &make_em(shift));
                wrapper.set_attribute("depth", &make_em(-shift));
            }
            wrapper.set_attribute("voffset", &make_em(shift));

            wrapper.into()
        })),
    });

    fns.insert("\\rule".into(), rule);
}
