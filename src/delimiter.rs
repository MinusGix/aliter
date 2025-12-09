use crate::{
    build_common::{self, make_span, VListElem, VListKern, VListParam, VListShiftChild},
    dom_tree::{CssStyle, HtmlNode, PathNode, Span, SvgChildNode, SvgNode, SymbolNode, WithHtmlDomNode},
    expander::Mode,
    font_metrics::{get_character_metrics, CharacterMetrics},
    font_metrics_data,
    style::{StyleId, SCRIPT_SCRIPT_STYLE, SCRIPT_STYLE, TEXT_STYLE},
    svg_geometry,
    symbols,
    tree::ClassList,
    unit::make_em,
    util::{char_code_for, find_assoc_data},
    Options,
};

fn get_metrics(symbol: &str, font: &str, mode: Mode) -> CharacterMetrics {
    let replace = symbols::SYMBOLS
        .get(mode, symbol)
        .map(|sym| sym.replace)
        .flatten();
    let replace = replace.unwrap_or(symbol);
    // TODO: don't unwrap/expect. This should be a result!

    // The first character is the only thing we really get the metrics for.
    let replace_char = replace.chars().nth(0).unwrap();
    get_character_metrics(replace_char, font, mode)
        .expect("Unsupported symbol and font size combination")
}

/// Delimiters that stack when they become too large
const STACK_LARGE_DELIMITERS: &'static [&'static str] = &[
    "(", "\\lparen", ")", "\\rparen", "[", "\\lbrack", "]", "\\rbrack", "\\{", "\\lbrace", "\\}",
    "\\rbrace", "\\lfloor", "\\rfloor", "\u{230a}", "\u{230b}", "\\lceil", "\\rceil", "\u{2308}",
    "\u{2309}", "\\surd",
];

/// Delimiters that always stack
#[allow(dead_code)]
const STACK_ALWAYS_DELIMITERS: &'static [&'static str] = &[
    "\\uparrow",
    "\\downarrow",
    "\\updownarrow",
    "\\Uparrow",
    "\\Downarrow",
    "\\Updownarrow",
    "|",
    "\\|",
    "\\vert",
    "\\Vert",
    "\\lvert",
    "\\rvert",
    "\\lVert",
    "\\rVert",
    "\\lgroup",
    "\\rgroup",
    "\u{27ee}",
    "\u{27ef}",
    "\\lmoustache",
    "\\rmoustache",
    "\u{23b0}",
    "\u{23b1}",
];

/// Delimiters that never stack
const STACK_NEVER_DELIMITERS: &'static [&'static str] = &[
    "<",
    ">",
    "\\langle",
    "\\rangle",
    "/",
    "\\backslash",
    "\\lt",
    "\\gt",
];

/// Swap the delim if needed
fn delim_swap(delim: &str) -> &str {
    match delim {
        "<" | "\\lt" | "\u{27e8}" => "\\langle",
        ">" | "\\gt" | "\u{27e9}" => "\\rangle",
        _ => delim,
    }
}

// pub(crate) fn sized_delim(
//     delim: &str,
//     size: usize,
//     options: &Options,
//     mode: Mode,
//     classes: ClassList,
// ) -> DomSpan {
//     let delim = delim_swap(delim);

//     // Sized delims are never centered
//     if STACK_LARGE_DELIMITERS.contains(&delim) || STACK_NEVER_DELIMITERS.contains(&delim) {
//         make_large_delim(delim, size, false, options, mode, classes)
//     } else if STACK_ALWAYS_DELIMITERS.contains(&delim) {
//         make_stacked_delim(
//             delim,
//             SIZE_TO_MAX_HEIGHT[size],
//             false,
//             options,
//             mode,
//             classes,
//         )
//     } else {
//         panic!("Illegal delimiter {:?}", delim)
//     }
// }

/// Metrics of the different sizes. Found by looking at TeX's output of
/// $\bigl| // \Bigl| \biggl| \Biggl| \showlists$
/// Used to create stacked delimiters of appropriate sizes in makeSizedDelim.
const SIZE_TO_MAX_HEIGHT: [f64; 5] = [0.0, 1.2, 1.8, 2.4, 3.0];

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum Delimiter {
    Small(StyleId),
    // TODO: limited type?
    /// 1 | 2 | 3 | 4
    Large(u8),
    Stack,
}

const STACK_NEVER_DELIMITER_SEQUENCE: [Delimiter; 7] = [
    Delimiter::Small(SCRIPT_SCRIPT_STYLE),
    Delimiter::Small(SCRIPT_STYLE),
    Delimiter::Small(TEXT_STYLE),
    Delimiter::Large(1),
    Delimiter::Large(2),
    Delimiter::Large(3),
    Delimiter::Large(4),
];

const STACK_ALWAYS_DELIMITER_SEQUENCE: [Delimiter; 4] = [
    Delimiter::Small(SCRIPT_SCRIPT_STYLE),
    Delimiter::Small(SCRIPT_STYLE),
    Delimiter::Small(TEXT_STYLE),
    Delimiter::Stack,
];

const STACK_LARGE_DELIMITER_SEQUENCE: [Delimiter; 8] = [
    Delimiter::Small(SCRIPT_SCRIPT_STYLE),
    Delimiter::Small(SCRIPT_STYLE),
    Delimiter::Small(TEXT_STYLE),
    Delimiter::Large(1),
    Delimiter::Large(2),
    Delimiter::Large(3),
    Delimiter::Large(4),
    Delimiter::Stack,
];

fn delim_size_to_font(size: u8) -> &'static str {
    match size {
        1 => "Size1-Regular",
        2 => "Size2-Regular",
        3 => "Size3-Regular",
        4 => "Size4-Regular",
        // TODO: We should verify whether this can actually occur via user input!
        // and what katex does!
        _ => unreachable!("Illegal delimiter size"),
    }
}
/// Get the font used in a delimiter based on what kind of delimiter it is.
fn delim_type_to_font(del: Delimiter) -> &'static str {
    match del {
        Delimiter::Small(_) => "Main-Regular",
        Delimiter::Large(size) => delim_size_to_font(size),
        Delimiter::Stack => "Size4-Regular",
    }
}

/// Traverse a sequence of delimiters to decide what kind of delimiter should be used to create a
/// delimiter of the given height+depth
pub(crate) fn traverse_sequence(
    delim: &str,
    height: f64,
    sequence: &[Delimiter],
    options: &Options,
) -> Delimiter {
    // TODO: ensure it never underflows

    // Here, we choose the index we should start at in the sequences. In smaller
    // sizes (which correspond to larger numbers in style.size) we start earlier
    // in the sequence. Thus, scriptscript starts at index 3-3=0, script starts
    // at index 3-2=1, text starts at 3-1=2, and display starts at min(2,3-0)=2
    let start = (3 - options.style.size()).min(2);
    for entry in sequence.iter().skip(start) {
        if let Delimiter::Stack = entry {
            // This is always the last delimiter
            break;
        }

        let font = delim_type_to_font(*entry);
        let metrics = get_metrics(delim, font, Mode::Math);
        let mut height_depth = metrics.height + metrics.depth;

        // Small delimiters are scaled down versions of the same font, so we
        // account for the style change size.

        if let Delimiter::Small(style) = entry {
            let new_options = options.having_base_style(Some(*style));
            let new_options = new_options.as_ref().unwrap_or(options);
            height_depth *= new_options.size_multiplier();
        }

        // Check if the delimiter at this size works for the given height
        if height_depth > height {
            return *entry;
        }
    }

    // We reached the end, so we just return the last sequence element
    *sequence.last().unwrap()
}

/// Make a delimiter of a given height+depth, with optional centering. Here, we traverse the
/// sequences, and create a delimiter that the sequence tells us to.
pub(crate) fn custom_sized_delim(
    delim: &str,
    height: f64,
    center: bool,
    options: &Options,
    mode: Mode,
    classes: ClassList,
) -> Span<HtmlNode> {
    let delim = delim_swap(delim);

    // Decide what sequence to use
    let sequence = if STACK_NEVER_DELIMITERS.contains(&delim) {
        &STACK_NEVER_DELIMITER_SEQUENCE as &[_]
    } else if STACK_LARGE_DELIMITERS.contains(&delim) {
        &STACK_LARGE_DELIMITER_SEQUENCE as &[_]
    } else {
        &STACK_ALWAYS_DELIMITER_SEQUENCE as &[_]
    };

    let delim_type = traverse_sequence(delim, height, sequence, options);

    // Get the delimter from font glyphs
    match delim_type {
        Delimiter::Small(style) => {
            small_delim(delim, style, center, options, mode, classes).using_html_node()
        }
        Delimiter::Large(size) => {
            large_delim(delim, size, center, options, mode, classes).using_html_node()
        }
        Delimiter::Stack => stacked_delim(delim, height, center, options, mode, classes),
    }
}

fn style_wrap<T: WithHtmlDomNode>(
    delim: T,
    to_style: StyleId,
    options: &Options,
    classes: ClassList,
) -> Span<T> {
    let new_options = options.having_base_style(Some(to_style));
    let new_options = new_options.as_ref().unwrap_or(options);

    let classes = {
        let mut classes = classes.clone();
        classes.extend(new_options.sizing_classes(options));
        classes
    };
    let children = vec![delim];
    let mut span = make_span(classes, children, Some(options), CssStyle::default());

    let delim_size_mult = new_options.size_multiplier() / options.size_multiplier();
    span.node.height *= delim_size_mult;
    span.node.depth *= delim_size_mult;
    span.node.max_font_size = options.size_multiplier();

    span
}

fn center_span<T: WithHtmlDomNode>(span: &mut Span<T>, options: &Options, style: StyleId) {
    let new_options = options.having_base_style(Some(style));
    let new_options = new_options.as_ref().unwrap_or(options);

    let shift = (1.0 - options.size_multiplier() / new_options.size_multiplier())
        * options.font_metrics().axis_height;

    span.node.classes.push("delimcenter".to_string());
    span.node.style.top = Some(make_em(shift).into());
    span.node.height -= shift;
    span.node.depth += shift;
}

/// Makes a small delimiter. This is a delimeter that comes in the Main-Regular font, but is
/// restyled to either be in textstyle, scriptstyle, or scriptscriptstyle.
fn small_delim(
    delim: &str,
    style: StyleId,
    center: bool,
    options: &Options,
    mode: Mode,
    classes: ClassList,
) -> Span<SymbolNode> {
    let text =
        build_common::make_symbol(delim, "Main-Regular", mode, Some(options), ClassList::new());
    let mut span = style_wrap(text, style, options, classes);
    if center {
        center_span(&mut span, options, style);
    }

    span
}

/// Builds a symbol in the given font size
fn mathrm_size(value: &str, size: u8, mode: Mode, options: &Options) -> SymbolNode {
    let font = delim_size_to_font(size);
    build_common::make_symbol(value, font, mode, Some(options), ClassList::new())
}

/// Makes a large delimiter. This is a delimiter that comes in the Size1, Size2, Size3, or Size4
/// fonts.  
/// It is always rendered in textstyle.
fn large_delim(
    delim: &str,
    size: u8,
    center: bool,
    options: &Options,
    mode: Mode,
    classes: ClassList,
) -> Span<Span<SymbolNode>> {
    let inner = mathrm_size(delim, size, mode, options);

    let span = make_span(
        vec!["delimsizing".to_string(), format!("size{}", size)],
        vec![inner],
        Some(options),
        CssStyle::default(),
    );
    let mut span = style_wrap(span, TEXT_STYLE, options, classes);

    if center {
        center_span(&mut span, options, TEXT_STYLE);
    }

    span
}

#[derive(Debug, Clone, Copy)]
enum GlyphFont {
    Size1Regular,
    Size4Regular,
}
impl GlyphFont {
    fn as_str(&self) -> &str {
        match self {
            GlyphFont::Size1Regular => "Size1-Regular",
            GlyphFont::Size4Regular => "Size4-Regular",
        }
    }
}
/// Make a span from a font glyph with the given offset and in the given font.  
/// This is used in [`make_stacked_delim`] to make the stacking pieces for the delimiter.
fn glyph_span(symbol: &str, font: GlyphFont, mode: Mode) -> VListElem<Span<Span<SymbolNode>>> {
    let size_class = match font {
        GlyphFont::Size1Regular => "delim-size1",
        GlyphFont::Size4Regular => "delim-size4",
    };

    let sym = build_common::make_symbol(symbol, font.as_str(), mode, None, ClassList::new());
    let span = make_span(ClassList::new(), vec![sym], None, CssStyle::default());
    let corner = make_span(
        vec!["delimsizinginner".to_string(), size_class.to_string()],
        vec![span],
        None,
        CssStyle::default(),
    );

    VListElem::new(corner)
}

fn make_inner(ch: char, height: f64, options: &Options) -> VListElem<Span<HtmlNode>> {
    // Create a span with inline SVG for the inner part of a tall stacked delimiter
    let size4_metrics = font_metrics_data::get_metric("Size4-Regular").unwrap();
    let size1_metrics = font_metrics_data::get_metric("Size1-Regular").unwrap();

    let code = char_code_for(ch);

    let width = if let Some(s4) = find_assoc_data(size4_metrics, code) {
        s4[4]
    } else {
        // TODO: don't unwrap! though the logic in katex doesn't appear to handle this case
        let s1 = find_assoc_data(size1_metrics, code).unwrap();
        s1[4]
    };

    let path = PathNode::new(
        ch.to_string(),
        Some(svg_geometry::inner_path(ch, (1000.0 * height).round() as u64)),
    );
    let svg_node = SvgNode::new(vec![SvgChildNode::Path(path)])
        .with_attribute("width", make_em(width))
        .with_attribute("height", make_em(height))
        .with_attribute("viewBox", format!("0 0 {} {}", (1000.0 * width).round(), (1000.0 * height).round()))
        .with_attribute("preserveAspectRatio", "xMinYMin slice");
        
    let span = make_span(
        vec![], 
        vec![HtmlNode::Svg(svg_node)], 
        Some(options), 
        CssStyle::default()
    );
    
    VListElem {
        elem: span,
        margin_left: None,
        margin_right: None,
        wrapper_classes: ClassList::new(),
        wrapper_style: CssStyle::default(),
    }
}

const LAP_IN_EMS: f64 = 0.008;
const LAP: VListShiftChild<Span<HtmlNode>> = VListShiftChild::Kern(VListKern(-1.0 * LAP_IN_EMS));
const VERTS: [&'static str; 4] = ["|", "\\lvert", "\\rvert", "\\vert"];
const DOUBLE_VERTS: [&'static str; 4] = ["\\|", "\\lVert", "\\rVert", "\\Vert"];

fn stacked_delim(
    delim: &str,
    height_total: f64,
    center: bool,
    options: &Options,
    mode: Mode,
    classes: ClassList,
) -> Span<HtmlNode> {
    let mut top: &str = delim;
    let mut middle: Option<&str> = None;
    let mut repeat: &str = delim;
    let mut bottom: &str = delim;
    // TODO: it sets middle to `null` while leaving the others as undefined.. is that special?

    let mut font = GlyphFont::Size1Regular;

    // We set the parts and font based on the symbol.
    match delim {
        "\\uparrow" => {
            bottom = "\u{23d0}";
            repeat = bottom;
        }
        "\\Uparrow" => {
            bottom = "\u{2016}";
            repeat = bottom;
        }
        "\\downarrow" => {
            top = "\u{23d0}";
            repeat = top;
        }
        "\\Downarrow" => {
            top = "\u{2016}";
            repeat = top;
        }
        "\\updownarrow" => {
            top = "\\uparrow";
            repeat = "\u{23d0}";
            bottom = "\\downarrow";
        }
        "\\Updownarrow" => {
            top = "\\Uparrow";
            repeat = "\u{2016}";
            bottom = "\\Downarrow";
        }
        "\\lbrack" | "[" => {
            top = "\u{23a1}";
            repeat = "\u{23a2}";
            bottom = "\u{23a3}";
            font = GlyphFont::Size4Regular;
        }
        "\\rbrack" | "]" => {
            top = "\u{23a4}";
            repeat = "\u{23a5}";
            bottom = "\u{23a6}";
            font = GlyphFont::Size4Regular;
        }
        "\\lfloor" | "\u{230a}" => {
            repeat = "\u{23a2}";
            top = repeat;
            bottom = "\u{23a3}";
            font = GlyphFont::Size4Regular;
        }
        "\\lceil" | "\u{2308}" => {
            top = "\u{23a1}";
            repeat = "\u{23a2}";
            bottom = repeat;
            font = GlyphFont::Size4Regular;
        }
        "\\rfloor" | "\u{230b}" => {
            repeat = "\u{23a5}";
            top = repeat;
            bottom = "\u{23a6}";
            font = GlyphFont::Size4Regular;
        }
        "\\rceil" | "\u{2309}" => {
            top = "\u{23a4}";
            repeat = "\u{23a5}";
            bottom = repeat;
            font = GlyphFont::Size4Regular;
        }
        "(" | "\\lparen" => {
            top = "\u{239b}";
            repeat = "\u{239c}";
            bottom = "\u{239d}";
            font = GlyphFont::Size4Regular;
        }
        ")" | "\\rparen" => {
            top = "\u{239e}";
            repeat = "\u{239f}";
            bottom = "\u{23a0}";
            font = GlyphFont::Size4Regular;
        }
        "\\{" | "\\lbrace" => {
            top = "\u{23a7}";
            middle = Some("\u{23a8}");
            bottom = "\u{23a9}";
            repeat = "\u{23aa}";
            font = GlyphFont::Size4Regular;
        }
        "\\}" | "\\rbrace" => {
            top = "\u{23ab}";
            middle = Some("\u{23ac}");
            bottom = "\u{23ad}";
            repeat = "\u{23aa}";
            font = GlyphFont::Size4Regular;
        }
        "\\lgroup" | "\u{27ee}" => {
            top = "\u{23a7}";
            bottom = "\u{23a9}";
            repeat = "\u{23aa}";
            font = GlyphFont::Size4Regular;
        }
        "\\rgroup" | "\u{27ef}" => {
            top = "\u{23ab}";
            bottom = "\u{23ad}";
            repeat = "\u{23aa}";
            font = GlyphFont::Size4Regular;
        }
        "\\lmoustache" | "\u{23b0}" => {
            top = "\u{23a7}";
            bottom = "\u{23ad}";
            repeat = "\u{23aa}";
            font = GlyphFont::Size4Regular;
        }
        "\\rmoustache" | "\u{23b1}" => {
            top = "\u{23ab}";
            bottom = "\u{23a9}";
            repeat = "\u{23aa}";
            font = GlyphFont::Size4Regular;
        }

        _ => {
            if VERTS.contains(&delim) {
                repeat = "\u{2223}";
            } else if DOUBLE_VERTS.contains(&delim) {
                repeat = "\u{2225}";
            }
        }
    }

    // Get the metrics of the four sections
    let top_metrics = get_metrics(top, font.as_str(), mode);
    let top_height_total = top_metrics.height + top_metrics.depth;

    let repeat_metrics = get_metrics(repeat, font.as_str(), mode);
    let repeat_height_total = repeat_metrics.height + repeat_metrics.depth;

    let bottom_metrics = get_metrics(bottom, font.as_str(), mode);
    let bottom_height_total = bottom_metrics.height + bottom_metrics.depth;

    let mut middle_height_total = 0.0;
    let mut middle_factor = 1.0;
    if let Some(middle) = middle {
        let middle_metrics = get_metrics(middle, font.as_str(), mode);
        middle_height_total = middle_metrics.height + middle_metrics.depth;
        // repeat symmetrically above and below middle
        middle_factor = 2.0;
    }

    // Calculate the minimal height that the delimiter can have.
    // It is at least the size of the top, bottom, and optional middle combined.
    let min_height = top_height_total + bottom_height_total + middle_height_total * middle_factor;

    // Compute the number of copies of the repeat symbol we will need
    let repeat_count = ((height_total - min_height) / (middle_factor * repeat_height_total))
        .ceil()
        .max(0.0);

    // Compute the total height of the delimiter including all the symbols
    let real_height_total = min_height + repeat_count * repeat_height_total * middle_factor;

    // The center of the delimiter is placed at the center of the axis. Note that in this context,
    // "center" means that the delimiter should be centered around the axis in the current style,
    // while normally it is centered around the axis in textstyle.
    let mut axis_height = options.font_metrics().axis_height;
    if center {
        axis_height *= options.size_multiplier();
    }

    assert_eq!(repeat.len(), 1);
    let repeat = repeat.chars().nth(0).unwrap();

    // Calculate the depth
    let depth = real_height_total / 2.0 - axis_height;

    // Now, we start building the pieces that go into the vlist.
    let mut stack: Vec<VListShiftChild<Span<HtmlNode>>> = Vec::new();

    // Add the bottom symbol
    let glyph = glyph_span(bottom, font, mode).map(Span::using_html_node);
    stack.push(VListShiftChild::Elem(glyph));
    stack.push(LAP);

    if let Some(middle) = middle {
        // When we have a middle bit, we need the omiddle and two repeated sections
        let inner_height =
            (real_height_total - top_height_total - bottom_height_total - middle_height_total)
                / 2.0
                + 2.0 * LAP_IN_EMS;

        let inner = make_inner(repeat, inner_height, options);
        let inner = VListShiftChild::Elem(inner);

        stack.push(inner);
        // Now insert the middle of the brace
        stack.push(LAP);

        let glyph = glyph_span(middle, font, mode).map(Span::using_html_node);
        let glyph = VListShiftChild::Elem(glyph);
        stack.push(glyph);

        stack.push(LAP);
        // TODO: We can just clone  the first inner
        let inner = make_inner(repeat, inner_height, options);
        let inner = VListShiftChild::Elem(inner);
        stack.push(inner);
    } else {
        // The middle section will be an SVG. Make it an extra 0.016em tall.
        // We'll overlap by 0.008em at top and bottom.
        let inner_height =
            real_height_total - top_height_total - bottom_height_total + 2.0 * LAP_IN_EMS;
        let inner = make_inner(repeat, inner_height, options);
        let inner = VListShiftChild::Elem(inner);
        stack.push(inner);
    }

    // Add the top symbol
    stack.push(LAP);

    let glyph = glyph_span(top, font, mode).map(Span::using_html_node);
    let glyph = VListShiftChild::Elem(glyph);
    stack.push(glyph);

    // Finally, build the vlist
    let new_options = options.having_base_style(Some(TEXT_STYLE));
    let new_options = new_options.as_ref().unwrap_or(options);

    let inner = build_common::make_v_list(
        VListParam::Bottom {
            amount: depth,
            children: stack,
        },
        new_options,
    );

    let span = make_span(
        vec!["delimsizing".to_string(), "mult".to_string()],
        vec![inner],
        Some(new_options),
        CssStyle::default(),
    );

    let span = style_wrap(span, TEXT_STYLE, options, classes);

    span.using_html_node()
}

// All surds have 0.08em of padding above the viniculum inside the SVG.
// That keeps browser span height rounding error from pinching the line.
/// Padding above the surd, measured inside the viewBox.
const VB_PAD: f64 = 80.0;
/// Padding, in ems, measured in the document.
const EM_PAD: f64 = 0.08;

// fn sqrt_svg(sqrt_name: &str, height: f64, view_box_height: f64, extra_viniculum: f64, options: &Options) -> SvgSpan {
//     let path = sqrt_path(sqrt_name, extra_viniculum, view_box_height);
//     let path_node = PathNode::new(sqrt_name, path);

//     let svg = SvgNode::new();
// }

fn sqrt_svg(
    sqrt_name: &str,
    height: f64,
    view_box_height: f64,
    extra_viniculum: f64,
    options: &Options,
) -> Span<HtmlNode> {
    let path = svg_geometry::sqrt_path(sqrt_name, extra_viniculum, view_box_height);
    let path_node = PathNode::new(sqrt_name.to_string(), Some(path));

    let svg = SvgNode::new(vec![SvgChildNode::Path(path_node)])
        .with_attribute("width", "400em")
        .with_attribute("height", make_em(height))
        .with_attribute("viewBox", format!("0 0 400000 {}", view_box_height))
        .with_attribute("preserveAspectRatio", "xMinYMin slice");

    make_span(
        vec!["hide-tail".to_string()],
        vec![HtmlNode::Svg(svg)],
        Some(options),
        CssStyle::default(),
    )
}

pub struct SqrtImageInfo {
    pub span: Span<HtmlNode>,
    pub rule_width: f64,
    pub advance_width: f64,
}

pub(crate) fn make_sqrt_image(height: f64, options: &Options) -> SqrtImageInfo {
    let new_options = options.having_base_sizing();
    let delim = traverse_sequence(
        "\\surd",
        height * new_options.size_multiplier(),
        &STACK_LARGE_DELIMITER_SEQUENCE,
        &new_options,
    );

    let mut size_multiplier = new_options.size_multiplier();
    let extra_viniculum = (options.min_rule_thickness.0
        - options.font_metrics().sqrt_rule_thickness)
        .max(0.0);

    let mut span;
    let span_height;
    let tex_height;
    let view_box_height;
    let advance_width;

    match delim {
        Delimiter::Small(_) => {
            view_box_height = 1000.0 + 1000.0 * extra_viniculum + VB_PAD;
            if height < 1.0 {
                size_multiplier = 1.0;
            } else if height < 1.4 {
                size_multiplier = 0.7;
            }
            span_height = (1.0 + extra_viniculum + EM_PAD) / size_multiplier;
            tex_height = (1.0 + extra_viniculum) / size_multiplier;
            span = sqrt_svg(
                "sqrtMain",
                span_height,
                view_box_height,
                extra_viniculum,
                options,
            );
            span.node.style.min_width = Some("0.853em".into());
            advance_width = 0.833 / size_multiplier;
        }
        Delimiter::Large(size) => {
            let size_idx = size as usize;
            view_box_height = (1000.0 + VB_PAD) * SIZE_TO_MAX_HEIGHT[size_idx];
            tex_height = (SIZE_TO_MAX_HEIGHT[size_idx] + extra_viniculum) / size_multiplier;
            span_height =
                (SIZE_TO_MAX_HEIGHT[size_idx] + extra_viniculum + EM_PAD) / size_multiplier;
            span = sqrt_svg(
                &format!("sqrtSize{}", size),
                span_height,
                view_box_height,
                extra_viniculum,
                options,
            );
            span.node.style.min_width = Some("1.02em".into());
            advance_width = 1.0 / size_multiplier;
        }
        Delimiter::Stack => {
            span_height = height + extra_viniculum + EM_PAD;
            tex_height = height + extra_viniculum;
            view_box_height = (1000.0 * height + extra_viniculum).floor() + VB_PAD;
            span = sqrt_svg(
                "sqrtTall",
                span_height,
                view_box_height,
                extra_viniculum,
                options,
            );
            span.node.style.min_width = Some("0.742em".into());
            advance_width = 1.056;
        }
    }

    span.node.height = tex_height;
    span.node.style.height = Some(make_em(span_height).into());

    SqrtImageInfo {
        span,
        rule_width: (options.font_metrics().sqrt_rule_thickness + extra_viniculum)
            * size_multiplier,
        advance_width,
    }
}

#[allow(dead_code)]
pub(crate) fn left_right_delim(
    delim: &str,
    height: f64,
    depth: f64,
    options: &Options,
    mode: Mode,
    classes: ClassList,
) -> Span<HtmlNode> {
    // We always center \left/\right delimeters, so the axis is always shifted
    let axis_height = options.font_metrics().axis_height * options.size_multiplier();

    // Taken from TeX source, tex.web, function make_left_right
    let delimiter_factor = 901.0;
    let delimiter_extend = 5.0 / options.font_metrics().pt_per_em;

    let max_dist_from_axis = (height - axis_height).max(depth + axis_height);

    // In real TeX, calculations are done using integral values which are
    // 65536 per pt, or 655360 per em. So, the division here truncates in
    // TeX but doesn't here, producing different results. If we wanted to
    // exactly match TeX's calculation, we could do
    //   Math.floor(655360 * maxDistFromAxis / 500) *
    //    delimiterFactor / 655360
    // (To see the difference, compare
    //    x^{x^{\left(\rule{0.1em}{0.68em}\right)}}
    // in TeX and KaTeX)
    let total_height = (max_dist_from_axis / 500.0 * delimiter_factor)
        .max(2.0 * max_dist_from_axis - delimiter_extend);

    custom_sized_delim(delim, total_height, true, options, mode, classes)
}
