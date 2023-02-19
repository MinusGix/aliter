use std::{borrow::Cow, sync::Arc};

use crate::{
    build_common::{
        self, make_span, make_span_s, make_symbol, math_sym, VListElem, VListElemShift, VListKern,
        VListParam, VListShiftChild,
    },
    dom_tree::{CssStyle, DocumentFragment, HtmlNode, WithHtmlDomNode},
    expander::Mode,
    html, mathml,
    mathml_tree::{self, MathNode, MathNodeType, MathmlNode},
    parse_node::{NodeInfo, OpNode, ParseNode, ParseNodeType},
    style::{StyleId, DISPLAY_STYLE},
    tree::ClassList,
    unit::make_em,
    util::{self, find_assoc_data},
    Options,
};

use super::{ord_argument, FunctionPropSpec, FunctionSpec, Functions};

const SINGLE_CHAR_BIG_OPS: &'static [(&'static str, &'static str)] = &[
    ("\u{220F}", "\\prod"),
    ("\u{2210}", "\\coprod"),
    ("\u{2211}", "\\sum"),
    ("\u{22c0}", "\\bigwedge"),
    ("\u{22c1}", "\\bigvee"),
    ("\u{22c2}", "\\bigcap"),
    ("\u{22c3}", "\\bigcup"),
    ("\u{2a00}", "\\bigodot"),
    ("\u{2a01}", "\\bigoplus"),
    ("\u{2a02}", "\\bigotimes"),
    ("\u{2a04}", "\\biguplus"),
    ("\u{2a06}", "\\bigsqcup"),
];

pub fn add_functions(fns: &mut Functions) {
    let prod = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Op, 0),
        handler: Box::new(|ctx, _, _| {
            let func_name: Option<Cow<'static, str>> = if ctx.func_name.len() == 1 {
                find_assoc_data(SINGLE_CHAR_BIG_OPS, ctx.func_name.as_ref())
                    .map(|x| *x)
                    .map(Cow::Borrowed)
            } else {
                Some(Cow::Owned(ctx.func_name.to_string()))
            };

            ParseNode::Op(OpNode {
                limits: true,
                always_handle_sup_sub: None,
                suppress_base_shift: None,
                parent_is_sup_sub: Some(false),
                symbol: true,
                name: func_name,
                body: None,
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });

    fns.insert_for_all_str(
        [
            "\\coprod",
            "\\bigvee",
            "\\bigwedge",
            "\\biguplus",
            "\\bigcap",
            "\\bigcup",
            "\\intop",
            "\\prod",
            "\\sum",
            "\\bigotimes",
            "\\bigoplus",
            "\\bigodot",
            "\\bigsqcup",
            "\\smallint",
            "\u{220F}",
            "\u{2210}",
            "\u{2211}",
            "\u{22c0}",
            "\u{22c1}",
            "\u{22c2}",
            "\u{22c3}",
            "\u{2a00}",
            "\u{2a01}",
            "\u{2a02}",
            "\u{2a04}",
            "\u{2a06}",
        ]
        .into_iter(),
        prod,
    );

    // TODO: this will duplicate the html and math builder, which is fine; but I don't know why katex does it??
    let mathop = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Op, 1).with_primitive(true),
        handler: Box::new(|ctx, args, _opt_args| {
            let body = args[0].clone();
            ParseNode::Op(OpNode {
                limits: false,
                always_handle_sup_sub: None,
                suppress_base_shift: None,
                parent_is_sup_sub: Some(false),
                symbol: false,
                name: None,
                body: Some(ord_argument(body)),
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });

    fns.insert(Cow::Borrowed("\\mathop"), mathop);

    // No limits, not symbols
    let spec = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Op, 0),
        handler: Box::new(|ctx, _, _| {
            ParseNode::Op(OpNode {
                limits: false,
                always_handle_sup_sub: None,
                suppress_base_shift: None,
                parent_is_sup_sub: Some(false),
                symbol: false,
                name: Some(Cow::Owned(ctx.func_name.to_string())),
                body: None,
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });

    fns.insert_for_all_str(
        [
            "\\arcsin", "\\arccos", "\\arctan", "\\arctg", "\\arcctg", "\\arg", "\\ch", "\\cos",
            "\\cosec", "\\cosh", "\\cot", "\\cotg", "\\coth", "\\csc", "\\ctg", "\\cth", "\\deg",
            "\\dim", "\\exp", "\\hom", "\\ker", "\\lg", "\\ln", "\\log", "\\sec", "\\sin",
            "\\sinh", "\\sh", "\\tan", "\\tanh", "\\tg", "\\th",
        ]
        .into_iter(),
        spec,
    );

    // Limits, not symbols
    let spec = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Op, 0),
        handler: Box::new(|ctx, _, _| {
            ParseNode::Op(OpNode {
                limits: true,
                always_handle_sup_sub: None,
                suppress_base_shift: None,
                parent_is_sup_sub: Some(false),
                symbol: false,
                name: Some(Cow::Owned(ctx.func_name.to_string())),
                body: None,
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });

    fns.insert_for_all_str(
        [
            "\\det", "\\gcd", "\\inf", "\\lim", "\\max", "\\min", "\\Pr", "\\sup",
        ]
        .into_iter(),
        spec,
    );

    // No limits, symbols
    let spec = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Op, 0),
        handler: Box::new(|ctx, _, _| {
            let func_name = if ctx.func_name.len() == 1 {
                Cow::Borrowed(match ctx.func_name.as_ref() {
                    "\u{222b}" => "\\int",
                    "\u{222c}" => "\\iint",
                    "\u{222d}" => "\\iiint",
                    "\u{222e}" => "\\oint",
                    "\u{222f}" => "\\oiint",
                    "\u{2230}" => "\\oiiint",
                    _ => unreachable!(),
                })
            } else {
                Cow::Owned(ctx.func_name.to_string())
            };

            ParseNode::Op(OpNode {
                limits: false,
                always_handle_sup_sub: None,
                suppress_base_shift: None,
                parent_is_sup_sub: Some(false),
                symbol: true,
                name: Some(func_name),
                body: None,
                info: NodeInfo::new_mode(ctx.parser.mode()),
            })
        }),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });

    fns.insert_for_all_str(
        [
            "\\int", "\\iint", "\\iiint", "\\oint", "\\oiint", "\\oiiint", "\u{222b}", "\u{222c}",
            "\u{222d}", "\u{222e}", "\u{222f}", "\u{2230}",
        ]
        .into_iter(),
        spec,
    );
}

#[cfg(feature = "html")]
fn html_builder(group: &ParseNode, options: &Options) -> HtmlNode {
    let mut sup_group = None;
    let mut sub_group = None;
    let mut has_limits = false;
    let group = match group {
        ParseNode::SupSub(sup_sub) => {
            // If we have limits, supsub will pass us its group to handle. Pull out the superscript
            // and subscript and set the group to the op its base.
            sup_group = sup_sub.sub.as_deref();
            sub_group = sup_sub.sup.as_deref();
            has_limits = true;
            let Some(ParseNode::Op(base)) = sup_sub.base.as_deref() else {
                panic!("Expected group to be an op");
            };

            base
        }
        ParseNode::Op(op) => op,
        _ => panic!("Expected group to be an op or supsub"),
    };

    let style = options.style;

    // Most symbol operators get larger in displaystyle (rule 13)
    let large = style.size() == DISPLAY_STYLE.size()
        && group.symbol
        && group.name != Some(Cow::Borrowed("\\smallint"));

    let mut group_name = group.name.as_deref().unwrap();
    let mut base = if group.symbol {
        // If this is a symbol, create the symbol
        let font_name = large.then_some("Size2-Regular").unwrap_or("Size1-Regular");

        let stash = if group_name == "\\oiint" || group_name == "\\oiiint" {
            // No font glyphs yet, so use a glyph w/o the oval.
            let stash = &group.name.as_deref().unwrap()[1..];
            group_name = if stash == "oiint" {
                "\\iint"
            } else {
                "\\iiint"
            };
            stash
        } else {
            ""
        };

        let classes = vec![
            "mop".to_string(),
            "op-symbol".to_string(),
            large
                .then_some("large-op")
                .unwrap_or("small-op")
                .to_string(),
        ];
        let base = make_symbol(group_name, font_name, Mode::Math, Some(options), classes);

        let base: HtmlNode = if !stash.is_empty() {
            // We're in \oiint or \oiiint. Overlay the oval.
            // let italic = base.italic;
            let oval = format!("{}Size{}", stash, large.then_some("2").unwrap_or("1"));
            let oval = build_common::static_svg(&oval, options);

            let children = vec![
                VListElemShift::new(HtmlNode::from(base), 0.0),
                VListElemShift::new(HtmlNode::from(oval), large.then_some(0.08).unwrap_or(0.0)),
            ];
            let mut base =
                build_common::make_v_list(VListParam::IndividualShift { children }, options);

            group_name = if stash == "oiint" {
                "\\oiint"
            } else {
                "\\oiiint"
            };

            base.node.classes.insert(0, "mop".to_string());
            // FIXME: KaTeX sets italic on on the base, but that field doesn't exist??
            // base.ital

            base.into()
        } else {
            base.into()
        };

        base.into()
    } else if let Some(body) = group.body.as_deref() {
        // If this is a list, compose that list
        let inner = html::build_expression(body, options, html::RealGroup::True, (None, None));

        if inner.len() == 1 && matches!(inner[0], HtmlNode::Symbol(_)) {
            let mut base = inner.into_iter().nth(0).unwrap();
            base.node_mut().classes.push("mop".to_string());
            base
        } else {
            make_span(
                vec!["mop".to_string()],
                inner,
                Some(options),
                CssStyle::default(),
            )
            .into()
        }
    } else {
        // Otherwise, this is a text operator. Build the text from the operator's name.
        let mut output = Vec::new();
        if let Some(name) = &group.name {
            for chr in name.chars().skip(1) {
                // TODO: don't allocate single char strings
                output.push(math_sym(
                    &chr.to_string(),
                    group.info.mode,
                    options,
                    ClassList::new(),
                ));
            }
        }

        make_span(
            vec!["mop".to_string()],
            output,
            Some(options),
            CssStyle::default(),
        )
        .into()
    };

    // If content of op is a single symbol, shift it vertically
    let mut base_shift = 0.0;
    let mut slant = 0.0;

    if (matches!(base, HtmlNode::Symbol(_)) || group_name == "\\oiint" || group_name == "\\oiiint")
        && matches!(group.suppress_base_shift, Some(false) | None)
    {
        // We suppress the shift of the base of \overset and \underset. Otherwise,
        // shift the symbol so its center lies on the axis (rule 13). It
        // appears that our fonts have the centers of the symbols already
        // almost on the axis, so these numbers are very small. Note we
        // don't actually apply this here, but instead it is used either in
        // the vlist creation or separately when there are no limits.
        base_shift =
            (base.node().height - base.node().depth) / 2.0 - options.font_metrics().axis_height;

        // TODO: the katex code accesses this field even when we aren't assured that it is a symbol node! Does this even have any meaning for spans?
        if let HtmlNode::Symbol(symbol) = &base {
            slant = symbol.italic;
        }
    }

    if has_limits {
        assemble_sup_sub(
            base, sup_group, sub_group, options, style, slant, base_shift,
        )
    } else {
        if base_shift != 0.0 {
            base.node_mut().style.position = Some(Cow::Borrowed("relative"));
            base.node_mut().style.top = Some(Cow::Owned(make_em(base_shift)));
        }

        base
    }
}

struct PartInfo {
    elem: HtmlNode,
    kern: f64,
}
pub(crate) fn assemble_sup_sub(
    base: HtmlNode,
    sup_group: Option<&ParseNode>,
    sub_group: Option<&ParseNode>,
    options: &Options,
    style: StyleId,
    slant: f64,
    base_shift: f64,
) -> HtmlNode {
    let base = make_span_s(ClassList::new(), vec![base]);

    let sub_is_single_character = sub_group.map(util::is_character_box).unwrap_or(false);

    let sup = if let Some(sup_group) = sup_group {
        let new_options = options.having_style(style.sup());
        let new_options = new_options.as_ref().unwrap_or(options);
        let elem = html::build_group(Some(sup_group), new_options, Some(options));

        let depth = elem.node().depth;
        Some(PartInfo {
            elem,
            kern: options
                .font_metrics()
                .big_op_spacing1
                .max(options.font_metrics().big_op_spacing3 - depth),
        })
    } else {
        None
    };

    let sub = if let Some(sub_group) = sub_group {
        let new_options = options.having_style(style.sub());
        let new_options = new_options.as_ref().unwrap_or(options);
        let elem = html::build_group(Some(sub_group), new_options, Some(options));

        let height = elem.node().height;
        Some(PartInfo {
            elem,
            kern: options
                .font_metrics()
                .big_op_spacing2
                .max(options.font_metrics().big_op_spacing4 - height),
        })
    } else {
        None
    };

    let has_sub = sub.is_some();

    let metrics = options.font_metrics();
    // Build the final group as a vlist of the possible subscript, base, and possible superscript.
    let final_group = if let (Some(sup), Some(sub)) = (&sup, &sub) {
        let bottom = metrics.big_op_spacing5
            + sub.elem.node().height
            + sub.elem.node().depth
            + sub.kern
            + base.node().depth
            + base_shift;

        let children = vec![
            VListShiftChild::Kern(VListKern(metrics.big_op_spacing5)),
            VListShiftChild::Elem(VListElem::new_margin_left(
                sub.elem.clone(),
                make_em(-slant),
            )),
            VListShiftChild::Kern(VListKern(sub.kern)),
            VListShiftChild::Elem(VListElem::new(base.into())),
            VListShiftChild::Kern(VListKern(sup.kern)),
            VListShiftChild::Elem(VListElem::new_margin_left(sup.elem.clone(), make_em(slant))),
            VListShiftChild::Kern(VListKern(metrics.big_op_spacing5)),
        ];

        build_common::make_v_list(
            VListParam::Bottom {
                amount: bottom,
                children,
            },
            options,
        )
    } else if let Some(sub) = sub {
        let top = base.node().height - base_shift;

        // Shift the limits by the slant of the symbol. Note
        // that we are supposed to shift the limits by 1/2 of the slant,
        // but since we are centering the limits adding a full slant of
        // margin will shift by 1/2 that.
        let children = vec![
            VListShiftChild::Kern(VListKern(metrics.big_op_spacing5)),
            VListShiftChild::Elem(VListElem::new_margin_left(sub.elem, make_em(-slant))),
            VListShiftChild::Kern(VListKern(sub.kern)),
            VListShiftChild::Elem(VListElem::new(base.into())),
        ];

        build_common::make_v_list(
            VListParam::Top {
                amount: top,
                children: children,
            },
            options,
        )
    } else if let Some(sup) = sup {
        let bottom = base.node().depth + base_shift;

        let children = vec![
            VListShiftChild::Elem(VListElem::new(base.into())),
            VListShiftChild::Kern(VListKern(sup.kern)),
            VListShiftChild::Elem(VListElem::new_margin_left(sup.elem, make_em(slant))),
            VListShiftChild::Kern(VListKern(metrics.big_op_spacing5)),
        ];

        build_common::make_v_list(
            VListParam::Bottom {
                amount: bottom,
                children,
            },
            options,
        )
    } else {
        // This case probably shouldn't occur (this would mean the
        // supsub was sending us a group with no superscript or
        // subscript) but be safe.
        return base.into();
    };

    let mut parts = vec![final_group];
    if has_sub && slant != 0.0 && !sub_is_single_character {
        // A negative margin-left was applied to the lower limit.
        // Avoid an overlap by placing a spacer on the left on the group.
        let mut spacer = make_span(
            vec!["mspace".to_string()],
            Vec::new(),
            Some(options),
            CssStyle::default(),
        );
        spacer.node.style.margin_right = Some(Cow::Owned(make_em(slant)));

        parts.insert(0, spacer);
    }

    make_span(
        vec!["mop".to_string(), "op-limits".to_string()],
        parts,
        Some(options),
        CssStyle::default(),
    )
    .into()
}

fn mathml_builder(group: &ParseNode, options: &Options) -> MathmlNode {
    let ParseNode::Op(group) = group else { unreachable!() };

    if group.symbol {
        // This is a symbol. Just add the symbol.
        let name = group.name.as_deref().unwrap();
        let text = mathml::make_text(name.to_string(), group.info.mode, None);
        let mut node = MathNode::new(MathNodeType::Mo, vec![text], ClassList::new());
        if name == "\\smallint" {
            node.set_attribute("largeop", "false");
        }

        node.into()
    } else if let Some(body) = group.body.as_deref() {
        // This is an operator with children. Add them
        let expr = mathml::build_expression(body, options, None);
        MathNode::new(MathNodeType::Mo, expr, ClassList::new()).into()
    } else {
        // This is a text operator. Add all of the character from the operator's name.
        let name = group.name.as_deref().unwrap();
        let text = mathml_tree::TextNode::new(name[1..].to_string());
        let node = MathNode::new(MathNodeType::Mo, vec![text], ClassList::new());

        // Append an <mo>&ApplyFunction;</mo>.
        // ref: https://www.w3.org/TR/REC-MathML/chap3_2.html#sec3.2.4
        let operator = mathml::make_text("\u{2061}".to_string(), Mode::Text, None);
        let operator = MathNode::new(MathNodeType::Mo, vec![operator], ClassList::new());

        let node = MathmlNode::from(node);
        let operator = MathmlNode::from(operator);

        if group.parent_is_sup_sub == Some(true) {
            MathNode::new(MathNodeType::MRow, vec![node, operator], ClassList::new()).into()
        } else {
            DocumentFragment::new(vec![node, operator]).into()
        }
    }
}
