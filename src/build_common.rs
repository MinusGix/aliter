use std::borrow::Cow;

use crate::{
    dom_tree::{CssStyle, HtmlDomNode, HtmlNode, Span, SymbolNode, WithHtmlDomNode},
    expander::Mode,
    font_metrics::{get_character_metrics, CharacterMetrics},
    symbols::{self, Font},
    tree::{ClassList, EmptyNode},
    unit::{self, make_em},
    Options,
};

pub(crate) struct LookupSymbol {
    pub value: String,
    pub metrics: Option<CharacterMetrics>,
}

pub(crate) fn lookup_symbol(value: &str, font: &str, mode: Mode) -> LookupSymbol {
    let value = symbols::SYMBOLS
        .get(mode, value)
        .and_then(|sym| sym.replace)
        .unwrap_or(value);

    let value_char = value.chars().nth(0).unwrap();

    let metrics = get_character_metrics(value_char, font, mode);

    LookupSymbol {
        value: value.to_string(),
        metrics,
    }
}

pub(crate) fn make_symbol(
    value: &str,
    font: &str,
    mode: Mode,
    options: Option<&Options>,
    classes: ClassList,
) -> SymbolNode {
    let LookupSymbol { value, metrics } = lookup_symbol(value, font, mode);

    let mut symbol_node = if let Some(metrics) = metrics {
        let italic = if mode == Mode::Text || options.map(|o| o.font == "mathit").unwrap_or(false) {
            metrics.italic
        } else {
            0.0
        };

        SymbolNode::new(
            value,
            Some(metrics.height),
            Some(metrics.depth),
            Some(italic),
            Some(metrics.skew),
            Some(metrics.width),
            classes,
            CssStyle::default(),
        )
    } else {
        // TODO: log warning
        SymbolNode::new_text_classes(value, classes)
    };

    if let Some(options) = options {
        symbol_node.node.max_font_size = options.size_multiplier();
        if options.style.is_tight() {
            symbol_node.node.classes.push("mtight".to_string());
        }

        if let Some(color) = options.get_color() {
            symbol_node.node.style.color = Some(color);
        }
    }

    symbol_node
}

/// Makes a symbol in Main-Regular or AMS-Regular.  
/// Used for rel, bin, open, close, inner and punct.
pub(crate) fn math_sym(
    value: &str,
    mode: Mode,
    options: &Options,
    classes: ClassList,
) -> SymbolNode {
    if options.font == "boldsymbol" && lookup_symbol(value, "Main-Bold", mode).metrics.is_some() {
        let mut classes = classes.clone();
        classes.push("mathbf".to_string());
        make_symbol(value, "Main-Bold", mode, Some(options), classes)
    } else if value == "\\"
        || symbols::SYMBOLS
            .get(mode, value)
            .map(|s| s.font == Font::Main)
            .unwrap_or(false)
    {
        make_symbol(value, "Main-Regular", mode, Some(options), classes)
    } else {
        make_symbol(value, "AMS-Regular", mode, Some(options), classes)
    }
}

// TODO: should this be put elsewhere?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrdType {
    MathOrd,
    TextOrd,
}

pub(crate) struct BoldSymbolInfo {
    font: &'static str,
    font_class: &'static str,
}

pub(crate) fn bold_symbol(value: &str, mode: Mode, typ: OrdType) -> BoldSymbolInfo {
    if typ != OrdType::TextOrd
        && lookup_symbol(value, "Math-BoldItalic", mode)
            .metrics
            .is_some()
    {
        BoldSymbolInfo {
            font: "Math-BoldItalic",
            font_class: "boldsymbol",
        }
    } else {
        // Some glyphs do not exist in Math-BoldItalic so we need to use Main-Bold instead.
        BoldSymbolInfo {
            font: "Main-Bold",
            font_class: "mathbf",
        }
    }
}

// TODO: make_ord

fn size_element_for_children<T: WithHtmlDomNode>(node: &mut HtmlDomNode, children: &[T]) {
    let mut height: f64 = 0.0;
    let mut depth: f64 = 0.0;
    let mut max_font_size: f64 = 0.0;

    for child in children {
        let child_node = child.node();
        height = height.max(child_node.height);
        depth = depth.max(child_node.depth);
        max_font_size = max_font_size.max(child_node.max_font_size);
    }

    node.height = height;
    node.depth = depth;
    node.max_font_size = max_font_size;
}

// TODO: Should these all just be on `Span`?
pub(crate) fn make_span<T: WithHtmlDomNode>(
    classes: ClassList,
    children: Vec<T>,
    options: Option<&Options>,
    style: CssStyle,
) -> Span<T> {
    let mut span = Span::new(classes, children, options, style);

    size_element_for_children(&mut span.node, &span.children);

    span
}
pub(crate) fn make_span_s<T: WithHtmlDomNode>(classes: ClassList, children: Vec<T>) -> Span<T> {
    make_span(classes, children, None, CssStyle::default())
}

pub(crate) fn make_empty_span(classes: ClassList) -> Span<EmptyNode> {
    Span::new(classes, Vec::new(), None, CssStyle::default())
}

pub(crate) fn make_line_span(
    class_name: &str,
    options: &Options,
    thickness: Option<f64>,
) -> Span<HtmlNode> {
    let mut line = make_span::<HtmlNode>(
        vec![class_name.to_string()],
        Vec::new(),
        Some(options),
        CssStyle::default(),
    );
    line.node.height = thickness
        .unwrap_or(options.font_metrics().default_rule_thickness)
        .max(options.min_rule_thickness.0);
    line.node.style.border_bottom_width = Some(Cow::Owned(unit::make_em(line.node.height)));
    line.node.max_font_size = 1.0;
    line
}

#[derive(Debug, Clone)]
pub(crate) struct VListElem<T> {
    pub(crate) elem: T,
    pub(crate) margin_left: Option<Cow<'static, str>>,
    pub(crate) margin_right: Option<Cow<'static, str>>,
    pub(crate) wrapper_classes: ClassList,
    pub(crate) wrapper_style: CssStyle,
}
impl<T: WithHtmlDomNode> VListElem<T> {
    pub(crate) fn new(elem: T) -> VListElem<T> {
        VListElem {
            elem,
            margin_left: None,
            margin_right: None,
            wrapper_classes: ClassList::default(),
            wrapper_style: CssStyle::default(),
        }
    }

    pub(crate) fn map<U: WithHtmlDomNode>(self, f: impl FnOnce(T) -> U) -> VListElem<U> {
        VListElem {
            elem: f(self.elem),
            margin_left: self.margin_left,
            margin_right: self.margin_right,
            wrapper_classes: self.wrapper_classes,
            wrapper_style: self.wrapper_style,
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct VListElemShift<T> {
    pub(crate) elem: VListElem<T>,
    pub(crate) shift: f64,
}
impl<T: WithHtmlDomNode> VListElemShift<T> {
    pub(crate) fn new(elem: T, shift: f64) -> VListElemShift<T> {
        VListElemShift {
            elem: VListElem::new(elem),
            shift,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct VListKern(pub(crate) f64);

#[derive(Debug, Clone)]
pub(crate) enum VListShiftChild<T> {
    Elem(VListElem<T>),
    Shift(VListElemShift<T>),
    Kern(VListKern),
}
impl<T: WithHtmlDomNode> VListShiftChild<T> {
    pub(crate) fn into_elem(self) -> Option<VListElem<T>> {
        match self {
            VListShiftChild::Elem(elem) => Some(elem),
            VListShiftChild::Shift(elem) => Some(elem.elem),
            VListShiftChild::Kern(_) => None,
        }
    }

    pub(crate) fn elem(&self) -> Option<&VListElem<T>> {
        match self {
            VListShiftChild::Elem(elem) => Some(elem),
            VListShiftChild::Shift(elem) => Some(&elem.elem),
            VListShiftChild::Kern(_) => None,
        }
    }
}

pub(crate) enum VListParam<T> {
    /// Where each child contains how much it should be shifted downward
    IndividualShift { children: Vec<VListElemShift<T>> },
    /// `amount` specifies the topmost point of the vlist
    /// NOTE: This should not have any shift entries
    Top {
        amount: f64,
        children: Vec<VListShiftChild<T>>,
    },
    /// `amount` specifies the bottommost point of the vlist
    /// NOTE: This should not have any shift entries
    Bottom {
        amount: f64,
        children: Vec<VListShiftChild<T>>,
    },
    /// positioned such that its baseline is `amount` away from the baseline of the first child which MUST be a `Elem`
    /// NOTE: This should not have any shift entries
    Shift {
        amount: f64,
        children: Vec<VListShiftChild<T>>,
    },
    /// NOTE: This should not have any shift entries
    /// Positioned so that its baseline is aligned with the baseline of the first child which must be a `Elem`  
    /// This is equivalent to `Shift` with `amount=0`
    FirstBaseLine { children: Vec<VListShiftChild<T>> },
}
impl<T: WithHtmlDomNode> VListParam<T> {
    fn into_children_and_depth(self) -> (Vec<VListShiftChild<T>>, f64) {
        match self {
            VListParam::IndividualShift { children } => {
                // This is implemented differently due to wanting to avoid requiring T: Clone
                let mut new_children = Vec::new();

                let depth = -children[0].shift - children[0].elem.elem.node().depth;
                let mut curr_pos = depth;

                let mut prev_height = 0.0;
                let mut prev_depth = 0.0;
                // Add in kerns to the list of params.children to get each element to be
                // shifted to the correct specified shift
                for (i, child) in children.into_iter().enumerate() {
                    let cur_height = child.elem.elem.node().height;
                    let cur_depth = child.elem.elem.node().depth;
                    if i == 0 {
                        new_children.push(VListShiftChild::Shift(child));
                    } else {
                        let diff = -child.shift - curr_pos - cur_depth;
                        let size = diff - (prev_height + prev_depth);

                        curr_pos = curr_pos + diff;

                        new_children.push(VListShiftChild::Kern(VListKern(size)));
                        new_children.push(VListShiftChild::Shift(child));
                    }

                    prev_height = cur_height;
                    prev_depth = cur_depth;
                }

                (new_children, depth)
            }
            VListParam::Top { amount, children } => {
                let mut bottom = amount;
                for child in children.iter() {
                    bottom -= match child {
                        VListShiftChild::Elem(elem) => {
                            elem.elem.node().height + elem.elem.node().depth
                        }
                        VListShiftChild::Kern(kern) => kern.0,
                        // This should be unreachable, but we don't want to panic
                        VListShiftChild::Shift(_) => 0.0,
                    };
                }
                (children, bottom)
            }
            VListParam::Bottom { amount, children } => (children, -amount),
            VListParam::Shift { amount, children } => {
                let first = match &children[0] {
                    VListShiftChild::Elem(elem) => elem,
                    // Shift shouldn't ever occur as a child
                    VListShiftChild::Shift(_) => unreachable!(),
                    VListShiftChild::Kern(_) => panic!("First child must have type 'elem'"),
                };
                let depth = -first.elem.node().depth - amount;

                (children, depth)
            }
            VListParam::FirstBaseLine { children } => {
                let first = match &children[0] {
                    VListShiftChild::Elem(elem) => elem,
                    // Shift shouldn't ever occur as a child
                    VListShiftChild::Shift(_) => unreachable!(),
                    VListShiftChild::Kern(_) => panic!("First child must have type 'elem'"),
                };
                let depth = -first.elem.node().depth;

                (children, depth)
            }
        }
    }
}

// TODO: This function could get rid of more of its boxes
/// Makes a vertical list by stacking elements and kerns on top of each other.  
/// Allows for many different ways of specifying the positioning method.
pub(crate) fn make_v_list<T: WithHtmlDomNode + Into<HtmlNode> + 'static>(
    params: VListParam<T>,
    _options: &Options,
) -> Span<HtmlNode> {
    let (children, depth) = params.into_children_and_depth();

    // Create a strut that is taller than any list item. The strut is added to
    // each item, where it will determine the item's baseline. Since it has
    // `overflow:hidden`, the strut's top edge will sit on the item's line box's
    // top edge and the strut's bottom edge will sit on the item's baseline,
    // with no additional line-height spacing. This allows the item baseline to
    // be positioned precisely without worrying about font ascent and
    // line-height.
    let mut pstrut_size: f64 = 0.0;
    for child in children.iter() {
        if let Some(elem) = child.elem() {
            pstrut_size = pstrut_size
                .max(elem.elem.node().max_font_size)
                .max(elem.elem.node().height);
        }
    }

    pstrut_size += 2.0;

    let mut pstrut = make_empty_span(vec!["pstrut".to_string()]);
    pstrut.node.style.height = Some(Cow::Owned(make_em(pstrut_size)));

    // Create a new list of actual children at the correct offsets
    let mut real_children: Vec<Span<HtmlNode>> = Vec::new();
    let mut min_pos = depth;
    let mut max_pos = depth;
    let mut curr_pos = depth;
    for child in children {
        if let VListShiftChild::Kern(kern) = &child {
            curr_pos += kern.0;
        }

        if let Some(elem) = child.into_elem() {
            let inner_elem = elem.elem;
            let classes = elem.wrapper_classes;
            let style = elem.wrapper_style;
            let margin_left = elem.margin_left;
            let margin_right = elem.margin_right;

            let i_height = inner_elem.node().height;
            let i_depth = inner_elem.node().depth;

            let pstrut = pstrut.clone().using_html_node();
            let mut child_wrap: Span<HtmlNode> =
                make_span(classes, vec![pstrut.into(), inner_elem.into()], None, style);
            child_wrap.node.style.top =
                Some(Cow::Owned(make_em(-pstrut_size - curr_pos - i_depth)));

            if let Some(margin_left) = margin_left {
                child_wrap.node.style.margin_left = Some(margin_left);
            }

            if let Some(margin_right) = margin_right {
                child_wrap.node.style.margin_right = Some(margin_right);
            }

            real_children.push(child_wrap);
            curr_pos += i_height + i_depth;
        }

        min_pos = min_pos.min(curr_pos);
        max_pos = max_pos.max(curr_pos);
    }

    // The vlist contents go in a table-cell with `vertical-align:bottom`.
    // This cell's bottom edge will determine the containing table's baseline
    // without overly expanding the containing line-box.
    let mut v_list = make_span_s(vec!["vlist".to_string()], real_children).using_html_node();
    v_list.node.style.height = Some(Cow::Owned(make_em(max_pos)));

    let rows: Vec<Span<Span<HtmlNode>>> = if min_pos < 0.0 {
        // We will define depth in an empty span with display: table-cell.
        // It should render with the height that we define. But Chrome, in
        // contenteditable mode only, treats that span as if it contains some
        // text content. And that min-height over-rides our desired height.
        // So we put another empty span inside the depth strut span.
        let empty_span = make_empty_span(ClassList::new());
        let mut depth_strut =
            make_span_s(vec!["vlist".to_string()], vec![empty_span]).using_html_node();
        depth_strut.node.style.height = Some(Cow::Owned(make_em(-min_pos)));

        // Safari wants the first row to have inline content; otherwise it puts the bottom of the *second* row on the baseline
        let symbol = SymbolNode::new_text("\u{200b}".to_string());
        let top_strut = make_span_s(vec!["vlist-s".to_string()], vec![symbol]).using_html_node();

        vec![
            make_span_s(vec!["vlist-r".to_string()], vec![v_list, top_strut]),
            make_span_s(vec!["vlist-r".to_string()], vec![depth_strut]),
        ]
    } else {
        vec![make_span_s(vec!["vlist-r".to_string()], vec![v_list])]
    };

    let rows_len = rows.len();
    let mut vtable = make_span_s(vec!["vlist-t".to_string()], rows);
    if rows_len == 2 {
        vtable.node.classes.push("vlist-t2".to_string());
    }

    vtable.node.height = max_pos;
    vtable.node.depth = -min_pos;

    vtable.using_html_node()
}
