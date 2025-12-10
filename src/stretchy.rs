//! Support for stretchy wide elements rendered from SVG files
//!
//! Many KaTeX stretchy wide elements use a long SVG image and an
//! overflow: hidden tactic to achieve a stretchy image while avoiding
//! distortion of arrowheads or brace corners.

use std::borrow::Cow;

#[cfg(feature = "html")]
use crate::{
    build_common::{make_span, make_svg_span},
    dom_tree::{CssStyle, HtmlNode, PathNode, SvgChildNode, SvgNode},
    unit::make_em,
    Options,
};

#[cfg(feature = "mathml")]
use crate::mathml_tree::{MathNode, MathNodeType, MathmlNode, TextNode};

/// Stretchy code points for MathML
pub fn stretchy_code_point(label: &str) -> &'static str {
    let label = label.strip_prefix('\\').unwrap_or(label);
    match label {
        "widehat" => "^",
        "widecheck" => "ˇ",
        "widetilde" => "~",
        "utilde" => "~",
        "overleftarrow" | "underleftarrow" | "xleftarrow" => "\u{2190}",
        "overrightarrow" | "underrightarrow" | "xrightarrow" => "\u{2192}",
        "underbrace" => "\u{23df}",
        "overbrace" => "\u{23de}",
        "overgroup" => "\u{23e0}",
        "undergroup" => "\u{23e1}",
        "overleftrightarrow" | "underleftrightarrow" | "xleftrightarrow" => "\u{2194}",
        "Overrightarrow" | "xRightarrow" => "\u{21d2}",
        "overleftharpoon" | "xleftharpoonup" => "\u{21bc}",
        "overrightharpoon" | "xrightharpoonup" => "\u{21c0}",
        "xLeftarrow" => "\u{21d0}",
        "xLeftrightarrow" => "\u{21d4}",
        "xhookleftarrow" => "\u{21a9}",
        "xhookrightarrow" => "\u{21aa}",
        "xmapsto" => "\u{21a6}",
        "xrightharpoondown" => "\u{21c1}",
        "xleftharpoondown" => "\u{21bd}",
        "xrightleftharpoons" => "\u{21cc}",
        "xleftrightharpoons" => "\u{21cb}",
        "xtwoheadleftarrow" => "\u{219e}",
        "xtwoheadrightarrow" => "\u{21a0}",
        "xlongequal" => "=",
        "xtofrom" | "xrightleftarrows" => "\u{21c4}",
        "xrightequilibrium" => "\u{21cc}",
        "xleftequilibrium" => "\u{21cb}",
        "cdrightarrow" => "\u{2192}",
        "cdleftarrow" => "\u{2190}",
        "cdlongequal" => "=",
        _ => "?",
    }
}

/// SVG path data: (paths, min_width, view_box_height, align)
/// Height is in SVG units (divide by 1000 for em)
#[derive(Clone, Copy)]
pub struct SvgImageData {
    pub paths: &'static [&'static str],
    pub min_width: f64,
    pub view_box_height: f64,
    pub align: Option<&'static str>,
}

pub fn get_svg_image_data(label: &str) -> Option<SvgImageData> {
    let label = label.strip_prefix('\\').unwrap_or(label);
    Some(match label {
        "overrightarrow" => SvgImageData {
            paths: &["rightarrow"],
            min_width: 0.888,
            view_box_height: 522.0,
            align: Some("xMaxYMin"),
        },
        "overleftarrow" => SvgImageData {
            paths: &["leftarrow"],
            min_width: 0.888,
            view_box_height: 522.0,
            align: Some("xMinYMin"),
        },
        "underrightarrow" => SvgImageData {
            paths: &["rightarrow"],
            min_width: 0.888,
            view_box_height: 522.0,
            align: Some("xMaxYMin"),
        },
        "underleftarrow" => SvgImageData {
            paths: &["leftarrow"],
            min_width: 0.888,
            view_box_height: 522.0,
            align: Some("xMinYMin"),
        },
        "xrightarrow" => SvgImageData {
            paths: &["rightarrow"],
            min_width: 1.469,
            view_box_height: 522.0,
            align: Some("xMaxYMin"),
        },
        "xleftarrow" => SvgImageData {
            paths: &["leftarrow"],
            min_width: 1.469,
            view_box_height: 522.0,
            align: Some("xMinYMin"),
        },
        "Overrightarrow" => SvgImageData {
            paths: &["doublerightarrow"],
            min_width: 0.888,
            view_box_height: 560.0,
            align: Some("xMaxYMin"),
        },
        "xRightarrow" => SvgImageData {
            paths: &["doublerightarrow"],
            min_width: 1.526,
            view_box_height: 560.0,
            align: Some("xMaxYMin"),
        },
        "xLeftarrow" => SvgImageData {
            paths: &["doubleleftarrow"],
            min_width: 1.526,
            view_box_height: 560.0,
            align: Some("xMinYMin"),
        },
        "overleftharpoon" | "xleftharpoonup" => SvgImageData {
            paths: &["leftharpoon"],
            min_width: 0.888,
            view_box_height: 522.0,
            align: Some("xMinYMin"),
        },
        "xleftharpoondown" => SvgImageData {
            paths: &["leftharpoondown"],
            min_width: 0.888,
            view_box_height: 522.0,
            align: Some("xMinYMin"),
        },
        "overrightharpoon" | "xrightharpoonup" => SvgImageData {
            paths: &["rightharpoon"],
            min_width: 0.888,
            view_box_height: 522.0,
            align: Some("xMaxYMin"),
        },
        "xrightharpoondown" => SvgImageData {
            paths: &["rightharpoondown"],
            min_width: 0.888,
            view_box_height: 522.0,
            align: Some("xMaxYMin"),
        },
        "xlongequal" => SvgImageData {
            paths: &["longequal"],
            min_width: 0.888,
            view_box_height: 334.0,
            align: Some("xMinYMin"),
        },
        "xtwoheadleftarrow" => SvgImageData {
            paths: &["twoheadleftarrow"],
            min_width: 0.888,
            view_box_height: 334.0,
            align: Some("xMinYMin"),
        },
        "xtwoheadrightarrow" => SvgImageData {
            paths: &["twoheadrightarrow"],
            min_width: 0.888,
            view_box_height: 334.0,
            align: Some("xMaxYMin"),
        },
        // Multi-part stretchy elements
        "overleftrightarrow" | "underleftrightarrow" => SvgImageData {
            paths: &["leftarrow", "rightarrow"],
            min_width: 0.888,
            view_box_height: 522.0,
            align: None,
        },
        "overbrace" => SvgImageData {
            paths: &["leftbrace", "midbrace", "rightbrace"],
            min_width: 1.6,
            view_box_height: 548.0,
            align: None,
        },
        "underbrace" => SvgImageData {
            paths: &["leftbraceunder", "midbraceunder", "rightbraceunder"],
            min_width: 1.6,
            view_box_height: 548.0,
            align: None,
        },
        "xleftrightarrow" => SvgImageData {
            paths: &["leftarrow", "rightarrow"],
            min_width: 1.75,
            view_box_height: 522.0,
            align: None,
        },
        "xLeftrightarrow" => SvgImageData {
            paths: &["doubleleftarrow", "doublerightarrow"],
            min_width: 1.75,
            view_box_height: 560.0,
            align: None,
        },
        "xrightleftharpoons" => SvgImageData {
            paths: &["leftharpoondownplus", "rightharpoonplus"],
            min_width: 1.75,
            view_box_height: 716.0,
            align: None,
        },
        "xleftrightharpoons" => SvgImageData {
            paths: &["leftharpoonplus", "rightharpoondownplus"],
            min_width: 1.75,
            view_box_height: 716.0,
            align: None,
        },
        "xhookleftarrow" => SvgImageData {
            paths: &["leftarrow", "righthook"],
            min_width: 1.08,
            view_box_height: 522.0,
            align: None,
        },
        "xhookrightarrow" => SvgImageData {
            paths: &["lefthook", "rightarrow"],
            min_width: 1.08,
            view_box_height: 522.0,
            align: None,
        },
        "overlinesegment" | "underlinesegment" => SvgImageData {
            paths: &["leftlinesegment", "rightlinesegment"],
            min_width: 0.888,
            view_box_height: 522.0,
            align: None,
        },
        "overgroup" => SvgImageData {
            paths: &["leftgroup", "rightgroup"],
            min_width: 0.888,
            view_box_height: 342.0,
            align: None,
        },
        "undergroup" => SvgImageData {
            paths: &["leftgroupunder", "rightgroupunder"],
            min_width: 0.888,
            view_box_height: 342.0,
            align: None,
        },
        "xmapsto" => SvgImageData {
            paths: &["leftmapsto", "rightarrow"],
            min_width: 1.5,
            view_box_height: 522.0,
            align: None,
        },
        "xtofrom" => SvgImageData {
            paths: &["leftToFrom", "rightToFrom"],
            min_width: 1.75,
            view_box_height: 528.0,
            align: None,
        },
        "xrightleftarrows" => SvgImageData {
            paths: &["baraboveleftarrow", "rightarrowabovebar"],
            min_width: 1.75,
            view_box_height: 901.0,
            align: None,
        },
        "xrightequilibrium" => SvgImageData {
            paths: &["baraboveshortleftharpoon", "rightharpoonaboveshortbar"],
            min_width: 1.75,
            view_box_height: 716.0,
            align: None,
        },
        "xleftequilibrium" => SvgImageData {
            paths: &["shortbaraboveleftharpoon", "shortrightharpoonabovebar"],
            min_width: 1.75,
            view_box_height: 716.0,
            align: None,
        },
        _ => return None,
    })
}

/// Create a MathML node for a stretchy element
#[cfg(feature = "mathml")]
pub fn mathml_node(label: &str) -> MathNode<MathmlNode> {
    let code_point = stretchy_code_point(label);
    let mut node = MathNode::new(
        MathNodeType::Mo,
        vec![MathmlNode::Text(TextNode::new(code_point.to_string()))],
        crate::tree::ClassList::new(),
    );
    node.set_attribute("stretchy", "true");
    node
}

/// Create an HTML span containing stretchy SVG element
#[cfg(feature = "html")]
pub fn svg_span(label: &str, options: &Options) -> HtmlNode {
    let label_stripped = label.strip_prefix('\\').unwrap_or(label);

    // Check for special widehat/widecheck/widetilde handling
    if matches!(label_stripped, "widehat" | "widecheck" | "widetilde" | "utilde") {
        // These need special handling based on content width - use defaults for now
        return build_widehat_svg(label_stripped, 1, options);
    }

    let Some(data) = get_svg_image_data(label) else {
        // Fallback - return empty span
        return make_span::<HtmlNode>(
            vec!["stretchy".to_string()],
            Vec::new(),
            Some(options),
            CssStyle::default(),
        ).into();
    };

    let view_box_width = 400000.0;
    let height = data.view_box_height / 1000.0;
    let num_paths = data.paths.len();

    if num_paths == 1 {
        // Single path with hide-tail technique
        let align = data.align.unwrap_or("xMinYMin");
        let path = PathNode::new(data.paths[0], None);
        let svg_node = SvgNode::new(vec![SvgChildNode::Path(path)])
            .with_attribute("width", "400em".to_string())
            .with_attribute("height", make_em(height))
            .with_attribute("viewBox", format!("0 0 {} {}", view_box_width, data.view_box_height))
            .with_attribute("preserveAspectRatio", format!("{} slice", align));

        let mut span = make_svg_span(vec!["hide-tail".to_string()], vec![svg_node], options);
        span.node.height = height;
        span.node.style.height = Some(Cow::Owned(make_em(height)));
        if data.min_width > 0.0 {
            span.node.style.min_width = Some(Cow::Owned(make_em(data.min_width)));
        }
        span.into()
    } else {
        // Multi-part stretchy element
        let (width_classes, aligns): (Vec<&str>, Vec<&str>) = if num_paths == 2 {
            (vec!["halfarrow-left", "halfarrow-right"], vec!["xMinYMin", "xMaxYMin"])
        } else if num_paths == 3 {
            (vec!["brace-left", "brace-center", "brace-right"], vec!["xMinYMin", "xMidYMin", "xMaxYMin"])
        } else {
            // Fallback
            return make_span::<HtmlNode>(
                vec!["stretchy".to_string()],
                Vec::new(),
                Some(options),
                CssStyle::default(),
            ).into();
        };

        let mut spans = Vec::new();
        for i in 0..num_paths {
            let path = PathNode::new(data.paths[i], None);
            let svg_node = SvgNode::new(vec![SvgChildNode::Path(path)])
                .with_attribute("width", "400em".to_string())
                .with_attribute("height", make_em(height))
                .with_attribute("viewBox", format!("0 0 {} {}", view_box_width, data.view_box_height))
                .with_attribute("preserveAspectRatio", format!("{} slice", aligns[i]));

            let mut span = make_svg_span(vec![width_classes[i].to_string()], vec![svg_node], options);
            span.node.style.height = Some(Cow::Owned(make_em(height)));
            spans.push(span.into());
        }

        let mut result = make_span::<HtmlNode>(
            vec!["stretchy".to_string()],
            spans,
            Some(options),
            CssStyle::default(),
        );
        result.node.height = height;
        result.node.style.height = Some(Cow::Owned(make_em(height)));
        if data.min_width > 0.0 {
            result.node.style.min_width = Some(Cow::Owned(make_em(data.min_width)));
        }
        result.into()
    }
}

/// Build widehat/widecheck/widetilde SVG based on content width
#[cfg(feature = "html")]
fn build_widehat_svg(label: &str, num_chars: usize, options: &Options) -> HtmlNode {
    let (view_box_width, view_box_height, height, path_name) = if num_chars > 5 {
        if label == "widehat" || label == "widecheck" {
            (2364.0, 420.0, 0.42, format!("{}4", label))
        } else {
            (2340.0, 312.0, 0.34, "tilde4".to_string())
        }
    } else {
        let img_index = [1, 1, 2, 2, 3, 3][num_chars.min(5)];
        if label == "widehat" || label == "widecheck" {
            let widths = [0.0, 1062.0, 2364.0, 2364.0, 2364.0];
            let heights = [0.0, 239.0, 300.0, 360.0, 420.0];
            let h = [0.0, 0.24, 0.3, 0.3, 0.36, 0.42];
            (widths[img_index], heights[img_index], h[img_index], format!("{}{}", label, img_index))
        } else {
            let widths = [0.0, 600.0, 1033.0, 2339.0, 2340.0];
            let heights = [0.0, 260.0, 286.0, 306.0, 312.0];
            let h = [0.0, 0.26, 0.286, 0.3, 0.306, 0.34];
            (widths[img_index], heights[img_index], h[img_index], format!("tilde{}", img_index))
        }
    };

    let path = PathNode::new(Cow::Owned(path_name), None);
    let svg_node = SvgNode::new(vec![SvgChildNode::Path(path)])
        .with_attribute("width", "100%".to_string())
        .with_attribute("height", make_em(height))
        .with_attribute("viewBox", format!("0 0 {} {}", view_box_width, view_box_height))
        .with_attribute("preserveAspectRatio", "none".to_string());

    let mut span = make_svg_span(Vec::new(), vec![svg_node], options);
    span.node.height = height;
    span.node.style.height = Some(Cow::Owned(make_em(height)));
    span.into()
}
