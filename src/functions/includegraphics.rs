use std::borrow::Cow;
use std::sync::Arc;

use crate::{
    parse_node::{IncludeGraphicsNode, NodeInfo, ParseNode, ParseNodeType},
    parser::ParseError,
    unit::{Em, Measurement},
    util::ArgType,
};

#[cfg(feature = "html")]
use crate::{
    dom_tree::{CssStyle, HtmlNode, Img},
    unit::{calculate_size, make_em},
    Options,
};

#[cfg(feature = "mathml")]
use crate::{
    mathml_tree::{MathNode, MathNodeType, MathmlNode},
    tree::ClassList,
};

use super::{FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let includegraphics = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_opt_args(ParseNodeType::IncludeGraphics, 1, 1)
            .with_allowed_in_text(true)
            .with_arg_types(&[ArgType::Raw, ArgType::Url] as &[ArgType]),
        handler: Box::new(includegraphics_handler),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });

    fns.insert(Cow::Borrowed("\\includegraphics"), includegraphics);
}

#[cfg(feature = "html")]
fn html_builder(group: &ParseNode, options: &Options) -> HtmlNode {
    let ParseNode::IncludeGraphics(group) = group else {
        panic!("Expected IncludeGraphics node");
    };

    let height = calculate_size(&group.height, options);
    let mut depth = 0.0;

    // Check if total_height has a non-zero value
    if let Measurement::Em(Em(val)) = &group.total_height {
        if *val > 0.0 {
            depth = calculate_size(&group.total_height, options) - height;
        }
    }

    let mut width = 0.0;
    if let Measurement::Em(Em(val)) = &group.width {
        if *val > 0.0 {
            width = calculate_size(&group.width, options);
        }
    }

    let mut style = CssStyle::default();
    style.height = Some(Cow::Owned(make_em(height + depth)));

    if width > 0.0 {
        style.width = Some(Cow::Owned(make_em(width)));
    }

    if depth > 0.0 {
        style.vertical_align = Some(Cow::Owned(make_em(-depth)));
    }

    let mut img = Img::new(group.src.clone(), group.alt.clone(), style);
    img.node.height = height;
    img.node.depth = depth;

    img.into()
}

#[cfg(feature = "mathml")]
fn mathml_builder(group: &ParseNode, options: &crate::Options) -> MathmlNode {
    let ParseNode::IncludeGraphics(group) = group else {
        panic!("Expected IncludeGraphics node");
    };

    let height = crate::unit::calculate_size(&group.height, options);
    let mut depth = 0.0;

    let mut node: MathNode<MathmlNode> = MathNode::new(MathNodeType::MGlyph, vec![], ClassList::new());
    node.set_attribute("alt", &group.alt);

    if let Measurement::Em(Em(val)) = &group.total_height {
        if *val > 0.0 {
            depth = crate::unit::calculate_size(&group.total_height, options) - height;
            node.set_attribute("valign", &crate::unit::make_em(-depth));
        }
    }

    node.set_attribute("height", &crate::unit::make_em(height + depth));

    if let Measurement::Em(Em(val)) = &group.width {
        if *val > 0.0 {
            let width = crate::unit::calculate_size(&group.width, options);
            node.set_attribute("width", &crate::unit::make_em(width));
        }
    }

    node.set_attribute("src", &group.src);

    node.into()
}

fn includegraphics_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let src = if let ParseNode::Url(url) = &args[0] {
        url.url.clone()
    } else {
        String::new()
    };

    // Check trust setting
    if !ctx.parser.conf.is_trusted("\\includegraphics", &src) {
        return Ok(ParseNode::Color(ctx.parser.format_unsupported_cmd("\\includegraphics")));
    }

    // Parse optional attributes like height, width, alt, totalheight
    let mut width = Measurement::Em(Em(0.0));
    let mut height = Measurement::Em(Em(0.9));  // Default height
    let mut total_height = Measurement::Em(Em(0.0));
    let mut alt = String::new();

    if let Some(Some(raw_node)) = opt_args.get(0) {
        if let ParseNode::Raw(raw) = raw_node {
            // Parse key=value pairs from attribute string
            // Strip trailing ] if present (parser artifact)
            let attr_string = raw.string.trim_end_matches(']');
            for attr in attr_string.split(',') {
                let parts: Vec<&str> = attr.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let val = parts[1].trim();
                    match key {
                        "alt" => alt = val.to_string(),
                        "width" => {
                            if let Ok(measurement) = parse_size(val) {
                                width = measurement;
                            }
                        }
                        "height" => {
                            if let Ok(measurement) = parse_size(val) {
                                height = measurement;
                            }
                        }
                        "totalheight" => {
                            if let Ok(measurement) = parse_size(val) {
                                total_height = measurement;
                            }
                        }
                        _ => {}  // Ignore unknown attributes
                    }
                }
            }
        }
    }

    // Default alt to filename if empty
    if alt.is_empty() {
        alt = src.clone();
        // Strip path and extension
        if let Some(pos) = alt.rfind('/') {
            alt = alt[pos + 1..].to_string();
        }
        if let Some(pos) = alt.rfind('\\') {
            alt = alt[pos + 1..].to_string();
        }
        if let Some(pos) = alt.rfind('.') {
            alt = alt[..pos].to_string();
        }
    }

    Ok(ParseNode::IncludeGraphics(IncludeGraphicsNode {
        alt,
        width,
        height,
        total_height,
        src,
        info: NodeInfo::new_mode(ctx.parser.mode()),
    }))
}

/// Parse a size value like "0.9em" or "10pt" into a Measurement
fn parse_size(s: &str) -> Result<Measurement, ()> {
    use regex::Regex;

    // Check for number-only (default to bp, per graphix package)
    let num_only = Regex::new(r"^[-+]?\s*(\d+(\.\d*)?|\.\d+)$").unwrap();
    if num_only.is_match(s) {
        let num: f64 = s.trim().parse().map_err(|_| ())?;
        // Convert bp to em (1bp = 1/72 inch, 1em ~= 12pt = 12/72 inch)
        return Ok(Measurement::Em(Em(num / 12.0)));
    }

    // Check for number with unit
    let num_unit = Regex::new(r"([-+]?)\s*(\d+(?:\.\d*)?|\.\d+)\s*([a-z]{2})").unwrap();
    if let Some(caps) = num_unit.captures(s) {
        let sign = if &caps[1] == "-" { -1.0 } else { 1.0 };
        let num: f64 = caps[2].parse().map_err(|_| ())?;
        let unit = &caps[3];

        // Convert common units to em
        let em_value = match unit {
            "em" => num,
            "pt" => num / 10.0,  // Approximate pt to em
            "px" => num / 16.0,  // Approximate px to em
            "bp" => num / 12.0,
            "mm" => num / 4.0,
            "cm" => num * 2.5,
            "in" => num * 6.0,
            _ => return Err(()),
        };

        return Ok(Measurement::Em(Em(sign * em_value)));
    }

    Err(())
}
