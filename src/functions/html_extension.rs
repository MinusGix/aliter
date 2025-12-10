use std::{collections::HashMap, sync::Arc};

use crate::{
    parse_node::{HtmlNode, NodeInfo, ParseNode, ParseNodeType},
    parser::ParseError,
    util::ArgType,
};

#[cfg(feature = "html")]
use crate::{
    build_common::make_span,
    dom_tree::{CssStyle, HtmlNode as DomHtmlNode},
    html::{self, RealGroup},
};

#[cfg(feature = "mathml")]
use crate::{
    mathml,
    mathml_tree::{MathNode, MathNodeType, MathmlNode},
    tree::ClassList,
};

use super::{ord_argument, FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    // \htmlId{id}{body}
    let html_id = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Html, 2)
            .with_allowed_in_text(true)
            .with_arg_types(&[ArgType::Raw, ArgType::Original] as &[ArgType]),
        handler: Box::new(html_id_handler),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });
    fns.insert("\\htmlId".into(), html_id);

    // \htmlClass{class}{body}
    let html_class = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Html, 2)
            .with_allowed_in_text(true)
            .with_arg_types(&[ArgType::Raw, ArgType::Original] as &[ArgType]),
        handler: Box::new(html_class_handler),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });
    fns.insert("\\htmlClass".into(), html_class);

    // \htmlStyle{style}{body}
    let html_style = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Html, 2)
            .with_allowed_in_text(true)
            .with_arg_types(&[ArgType::Raw, ArgType::Original] as &[ArgType]),
        handler: Box::new(html_style_handler),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });
    fns.insert("\\htmlStyle".into(), html_style);

    // \htmlData{key=value, ...}{body}
    let html_data = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::Html, 2)
            .with_allowed_in_text(true)
            .with_arg_types(&[ArgType::Raw, ArgType::Original] as &[ArgType]),
        handler: Box::new(html_data_handler),
        #[cfg(feature = "html")]
        html_builder: Some(Box::new(html_builder)),
        #[cfg(feature = "mathml")]
        mathml_builder: Some(Box::new(mathml_builder)),
    });
    fns.insert("\\htmlData".into(), html_data);
}

fn html_id_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    // Check trust setting
    if !ctx.parser.conf.trust {
        return Ok(ParseNode::Color(ctx.parser.format_unsupported_cmd("\\htmlId")));
    }

    let id = if let ParseNode::Raw(raw) = &args[0] {
        raw.string.clone()
    } else {
        String::new()
    };

    let body = args[1].clone();

    let mut attributes = HashMap::new();
    attributes.insert("id".to_string(), id);

    Ok(ParseNode::Html(HtmlNode {
        attributes,
        body: ord_argument(body),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    }))
}

fn html_class_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    // Check trust setting
    if !ctx.parser.conf.trust {
        return Ok(ParseNode::Color(ctx.parser.format_unsupported_cmd("\\htmlClass")));
    }

    let class = if let ParseNode::Raw(raw) = &args[0] {
        raw.string.clone()
    } else {
        String::new()
    };

    let body = args[1].clone();

    let mut attributes = HashMap::new();
    attributes.insert("class".to_string(), class);

    Ok(ParseNode::Html(HtmlNode {
        attributes,
        body: ord_argument(body),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    }))
}

fn html_style_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    // Check trust setting
    if !ctx.parser.conf.trust {
        return Ok(ParseNode::Color(ctx.parser.format_unsupported_cmd("\\htmlStyle")));
    }

    let style = if let ParseNode::Raw(raw) = &args[0] {
        raw.string.clone()
    } else {
        String::new()
    };

    let body = args[1].clone();

    let mut attributes = HashMap::new();
    attributes.insert("style".to_string(), style);

    Ok(ParseNode::Html(HtmlNode {
        attributes,
        body: ord_argument(body),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    }))
}

fn html_data_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    // Check trust setting
    if !ctx.parser.conf.trust {
        return Ok(ParseNode::Color(ctx.parser.format_unsupported_cmd("\\htmlData")));
    }

    let data_str = if let ParseNode::Raw(raw) = &args[0] {
        raw.string.clone()
    } else {
        String::new()
    };

    let body = args[1].clone();

    // Parse key=value pairs
    let mut attributes = HashMap::new();
    for pair in data_str.split(',') {
        let parts: Vec<&str> = pair.splitn(2, '=').collect();
        if parts.len() == 2 {
            let key = format!("data-{}", parts[0].trim());
            let value = parts[1].trim().to_string();
            attributes.insert(key, value);
        }
    }

    Ok(ParseNode::Html(HtmlNode {
        attributes,
        body: ord_argument(body),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    }))
}

#[cfg(feature = "html")]
fn html_builder(group: &ParseNode, options: &crate::Options) -> DomHtmlNode {
    let ParseNode::Html(group) = group else {
        panic!("Expected Html node");
    };

    let elements = html::build_expression(&group.body, options, RealGroup::False, (None, None));

    // Determine the classes for the span
    let classes = if let Some(class_value) = group.attributes.get("class") {
        vec![class_value.clone()]
    } else {
        Vec::new()
    };

    let mut span = make_span(
        classes,
        elements,
        Some(options),
        CssStyle::default(),
    );

    // Apply non-class attributes
    for (key, value) in &group.attributes {
        match key.as_str() {
            "id" => {
                span.attributes.insert("id".to_string(), value.clone());
            }
            "class" => {
                // Already handled above
            }
            "style" => {
                span.attributes.insert("style".to_string(), value.clone());
            }
            _ if key.starts_with("data-") => {
                span.attributes.insert(key.clone(), value.clone());
            }
            _ => {}
        };
    }

    span.into()
}

#[cfg(feature = "mathml")]
fn mathml_builder(group: &ParseNode, options: &crate::Options) -> MathmlNode {
    let ParseNode::Html(group) = group else {
        panic!("Expected Html node");
    };

    let inner = mathml::build_expression(&group.body, options, None);
    let mut node: MathNode<MathmlNode> = MathNode::new(MathNodeType::MRow, inner, ClassList::new());

    // Apply attributes (MathML doesn't support all HTML attributes but we can try)
    for (key, value) in &group.attributes {
        match key.as_str() {
            "id" => node.set_attribute("id", value),
            "class" => node.set_attribute("class", value),
            "style" => node.set_attribute("style", value),
            _ if key.starts_with("data-") => node.set_attribute(key, value),
            _ => {}
        }
    }

    node.into()
}
