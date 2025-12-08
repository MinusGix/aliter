use std::{borrow::Cow, sync::Arc};

use crate::{
    array::{AlignSpec, ColSeparationType},
    environments::{EnvironmentContext, EnvironmentSpec, Environments},
    expander::{BreakToken, Mode},
    functions::FunctionPropSpec,
    macr::MacroReplace,
    lexer::Token,
    parse_node::{
        ArrayNode, ArrayTag, LeftRightNode, NodeInfo, OrdGroupNode, ParseNode, StylingNode,
    },
    parser::{ParseError, Parser},
    util::Style,
};

fn get_hlines(parser: &mut Parser) -> Result<Vec<bool>, ParseError> {
    let mut lines = Vec::new();
    parser.consume_spaces()?;
    loop {
        let next = parser.fetch()?.content.clone();
        if next == "\\hline" || next == "\\hdashline" {
            parser.consume();
            lines.push(next == "\\hdashline");
            parser.consume_spaces()?;
        } else {
            break;
        }
    }
    Ok(lines)
}

fn d_cell_style(env_name: &str) -> Style {
    if env_name.starts_with('d') {
        Style::Display
    } else {
        Style::Text
    }
}

#[derive(Debug, Clone, Default)]
struct ArrayOpts {
    hskip_before_and_after: Option<bool>,
    add_jot: Option<bool>,
    cols: Option<Vec<AlignSpec>>,
    array_stretch: Option<f64>,
    col_separation_type: Option<ColSeparationType>,
    single_row: bool,
    empty_single_row: bool,
    max_num_cols: Option<usize>,
    leq_no: Option<bool>,
    is_cd: Option<bool>,
    tags: Option<Vec<ArrayTag>>,
    auto_tag: Option<bool>,
}

fn wrap_cell(body: Vec<ParseNode>, style: Style, mode: Mode) -> ParseNode {
    let ord = ParseNode::OrdGroup(OrdGroupNode {
        body,
        semi_simple: None,
        info: NodeInfo::new_mode(mode),
    });
    ParseNode::Styling(StylingNode {
        style,
        body: vec![ord],
        info: NodeInfo::new_mode(mode),
    })
}

fn array_from_opts(
    parser: &mut Parser,
    opts: ArrayOpts,
    style: Style,
) -> Result<ArrayNode, ParseError> {
    parser.gullet.begin_group();
    if !opts.single_row {
        parser
            .gullet
            .macros
            .set_back_macro("\\cr", Some(Arc::new(MacroReplace::Text("\\\\\\relax".to_string()))));
    }
    parser.gullet.begin_group();

    let mut array_stretch = if let Some(val) = opts.array_stretch {
        val
    } else if let Some(val) = parser.gullet.expand_macro_as_text("\\arraystretch")? {
        val.parse::<f64>().unwrap_or(1.0).max(0.0)
    } else {
        1.0
    };
    if array_stretch <= 0.0 {
        array_stretch = 1.0;
    }

    let auto_tag = opts.auto_tag;
    let mut tags = opts.tags.or_else(|| auto_tag.map(|_| Vec::new()));
    let mut begin_row = |parser: &mut Parser| {
        if let Some(true) = auto_tag {
            parser.gullet.macros.set_back_macro(
                "\\@eqnsw",
                Some(Arc::new(MacroReplace::Text("1".to_string()))),
            );
        }
    };
    let mut end_row =
        |parser: &mut Parser, tags: &mut Option<Vec<ArrayTag>>| -> Result<(), ParseError> {
            if let Some(tags_vec) = tags.as_mut() {
                if parser.gullet.macros.contains_back_macro("\\df@tag") {
                    let tag = parser.sub_parse(std::iter::once(Token::new_text("\\df@tag")))?;
                    tags_vec.push(ArrayTag::Tag(tag));
                    parser
                        .gullet
                        .macros
                        .set_global_back_macro("\\df@tag".to_string(), None);
                } else if let Some(auto) = auto_tag {
                    let eqnsw = parser
                        .gullet
                        .macros
                        .get_back_macro("\\@eqnsw")
                        .and_then(|m| match m.as_ref() {
                            MacroReplace::Text(t) => Some(t == "1"),
                            _ => None,
                        })
                        .unwrap_or(false);
                    tags_vec.push(ArrayTag::Boolean(auto && eqnsw));
                }
            }
            Ok(())
        };

    let mut body: Vec<Vec<ParseNode>> = vec![Vec::new()];
    let mut row_gaps = Vec::new();
    let mut h_lines_before_row = Vec::new();

    begin_row(parser);
    h_lines_before_row.push(get_hlines(parser)?);

    loop {
        let break_token = if opts.single_row {
            BreakToken::End
        } else {
            BreakToken::DoubleBackslash
        };
        let cell_body = parser.dispatch_parse_expression(false, Some(break_token))?;
        parser.gullet.end_group();
        parser.gullet.begin_group();

        let cell = wrap_cell(cell_body, style, parser.mode());
        let body_len = body.len();
        let row = body.last_mut().unwrap();
        row.push(cell);
        let row_empty_single = opts.empty_single_row
            && row.len() == 1
            && matches!(&row[0], ParseNode::Styling(StylingNode { body, .. }) if body.is_empty());

        drop(row);

        let next = parser.fetch()?.content.clone();
        if next == "&" {
            if let Some(max) = opts.max_num_cols {
                if body.last().map(|r| r.len()).unwrap_or(0) == max
                    && (opts.single_row || opts.col_separation_type.is_some())
                {
                    return Err(ParseError::Expected);
                }
            }
            parser.consume();
        } else if next == "\\end" {
            if h_lines_before_row.len() < body_len + 1 {
                h_lines_before_row.push(Vec::new());
            }
            end_row(parser, &mut tags)?;
            if row_empty_single {
                body.pop();
            }
            break;
        } else if next == "\\\\" {
            parser.consume();
            let size = if parser.gullet.future()?.content != " " {
                parser.parse_size_group(true)?
            } else {
                None
            };
            row_gaps.push(size.map(|s| s.value));
            end_row(parser, &mut tags)?;
            h_lines_before_row.push(get_hlines(parser)?);
            body.push(Vec::new());
            begin_row(parser);
        } else {
            eprintln!("array_from_opts: unexpected token {}", next);
            return Err(ParseError::Expected);
        }
    }

    parser.gullet.end_group();
    parser.gullet.end_group();

    Ok(ArrayNode {
        body,
        col_separation_type: opts.col_separation_type,
        h_skip_before_and_after: opts.hskip_before_and_after,
        add_jot: opts.add_jot,
        cols: opts.cols,
        array_stretch,
        row_gaps,
        h_lines_before_row,
        tags,
        leq_no: opts.leq_no,
        is_cd: opts.is_cd,
        info: NodeInfo::new_mode(parser.mode()),
    })
}

fn arg_to_chars(arg: &ParseNode) -> Result<Vec<char>, ParseError> {
    let normalize = |ch: char| match ch {
        '\u{2223}' | '\u{2225}' => '|',
        _ => ch,
    };
    if let Some(text) = arg.text() {
        return Ok(text.chars().map(normalize).collect());
    }

    if let ParseNode::OrdGroup(ord) = arg {
        let mut out = Vec::new();
        for node in &ord.body {
            if let Some(text) = node.text() {
                out.extend(text.chars().map(normalize));
            } else {
                return Err(ParseError::Expected);
            }
        }
        Ok(out)
    } else {
        Err(ParseError::Expected)
    }
}

fn align_spec_from_arg(arg: &ParseNode) -> Result<Vec<AlignSpec>, ParseError> {
    let chars = arg_to_chars(arg)?;
    let mut cols = Vec::new();
    for ch in chars {
        match ch {
            'l' | 'c' | 'r' => cols.push(AlignSpec::Align {
                align: ch.to_string().into(),
                pre_gap: None,
                post_gap: None,
            }),
            '|' => cols.push(AlignSpec::Separator("|".into())),
            ':' => cols.push(AlignSpec::Separator(":".into())),
            _ => return Err(ParseError::Expected),
        }
    }
    Ok(cols)
}

fn subarray_align_spec(arg: &ParseNode) -> Result<Vec<AlignSpec>, ParseError> {
    let chars = arg_to_chars(arg)?;
    if chars.len() != 1 {
        return Err(ParseError::Expected);
    }
    let ch = chars[0];
    if ch != 'l' && ch != 'c' {
        return Err(ParseError::Expected);
    }
    Ok(vec![AlignSpec::Align {
        align: ch.to_string().into(),
        pre_gap: None,
        post_gap: None,
    }])
}

fn validate_display(ctx: &EnvironmentContext) -> Result<(), ParseError> {
    if !ctx.parser.conf.display_mode {
        return Err(ParseError::Expected);
    }
    Ok(())
}

fn aligned_handler(
    mut ctx: EnvironmentContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    if !ctx.env_name.contains("ed") {
        validate_display(&ctx)?;
    }

    let mut opts = ArrayOpts::default();
    opts.add_jot = Some(true);
    opts.col_separation_type = if ctx.env_name.contains("at") {
        Some(ColSeparationType::AlignAt)
    } else {
        Some(ColSeparationType::Align)
    };
    opts.empty_single_row = true;
    opts.max_num_cols = if ctx.env_name == "split" { Some(2) } else { None };
    opts.leq_no = Some(ctx.parser.conf.leq_no);
    opts.auto_tag = if ctx.env_name == "split" {
        None
    } else if ctx.env_name.contains("ed") {
        None
    } else {
        Some(!ctx.env_name.contains('*'))
    };

    let mut res = array_from_opts(ctx.parser, opts, Style::Display)?;

    let mut num_cols = 0usize;
    if let Some(arg0) = args.get(0) {
        let text: String = arg_to_chars(arg0)?.into_iter().collect();
        if !text.is_empty() {
            num_cols = text.parse::<usize>().map_err(|_| ParseError::Expected)?;
            num_cols *= 2;
        }
    }
    let is_aligned = num_cols == 0;
    if is_aligned {
        num_cols = res.body.iter().map(|r| r.len()).max().unwrap_or(0);
    } else {
        for row in &res.body {
            if row.len() > num_cols {
                return Err(ParseError::Expected);
            }
        }
    }

    let mut cols = Vec::new();
    for i in 0..num_cols {
        let align = if i % 2 == 0 { "r" } else { "l" };
        let pregap = if i > 0 && is_aligned && i % 2 == 0 {
            Some(1.0)
        } else {
            None
        };
        cols.push(AlignSpec::Align {
            align: align.to_string().into(),
            pre_gap: pregap,
            post_gap: Some(0.0),
        });
    }
    res.cols = Some(cols);
    res.col_separation_type = if is_aligned {
        Some(ColSeparationType::Align)
    } else {
        Some(ColSeparationType::AlignAt)
    };

    Ok(ParseNode::Array(res))
}

fn matrix_handler(
    mut ctx: EnvironmentContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let mut col_align = "c".to_string();
    if ctx.env_name.ends_with('*') {
        ctx.parser.consume_spaces()?;
        if ctx.parser.fetch()?.content == "[" {
            ctx.parser.consume();
            ctx.parser.consume_spaces()?;
            let tok = ctx.parser.fetch()?.content.clone();
            if tok.len() != 1 || !"lcr".contains(tok.as_ref()) {
                return Err(ParseError::Expected);
            }
            col_align = tok.to_string();
            ctx.parser.consume();
            ctx.parser.consume_spaces()?;
            ctx.parser.expect("]", true)?;
        }
    }

    let res = array_from_opts(
        ctx.parser,
        ArrayOpts {
            hskip_before_and_after: Some(false),
            cols: Some(vec![AlignSpec::Align {
                align: col_align.clone().into(),
                pre_gap: None,
                post_gap: None,
            }]),
            ..Default::default()
        },
        d_cell_style(&ctx.env_name),
    )?;

    let num_cols = res.body.iter().map(|r| r.len()).max().unwrap_or(0);
    let cols = Some(
        (0..num_cols)
            .map(|_| AlignSpec::Align {
                align: col_align.clone().into(),
                pre_gap: None,
                post_gap: None,
            })
            .collect(),
    );

    let wrapped = if let Some((left, right)) = match ctx.env_name.trim_end_matches('*') {
        "pmatrix" => Some(("(".to_string(), ")".to_string())),
        "bmatrix" => Some(("[".to_string(), "]".to_string())),
        "Bmatrix" => Some(("\\{".to_string(), "\\}".to_string())),
        "vmatrix" => Some(("|".to_string(), "|".to_string())),
        "Vmatrix" => Some(("\\Vert".to_string(), "\\Vert".to_string())),
        "matrix" => None,
        _ => None,
    } {
        ParseNode::LeftRight(LeftRightNode {
            left,
            right,
            right_color: None,
            body: vec![ParseNode::Array(ArrayNode { cols, ..res })],
            info: NodeInfo::new_mode(ctx.mode),
        })
    } else {
        let mut res = res;
        res.cols = cols;
        ParseNode::Array(res)
    };

    Ok(wrapped)
}

fn array_handler(
    mut ctx: EnvironmentContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let cols = align_spec_from_arg(&args[0])?;
    let res = array_from_opts(
        ctx.parser,
        ArrayOpts {
            hskip_before_and_after: Some(true),
            cols: Some(cols.clone()),
            max_num_cols: Some(cols.len()),
            ..Default::default()
        },
        d_cell_style(&ctx.env_name),
    )?;
    Ok(ParseNode::Array(res))
}

fn smallmatrix_handler(
    mut ctx: EnvironmentContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let mut res = array_from_opts(
        ctx.parser,
        ArrayOpts {
            array_stretch: Some(0.5),
            ..Default::default()
        },
        Style::Script,
    )?;
    res.col_separation_type = Some(ColSeparationType::Small);
    Ok(ParseNode::Array(res))
}

fn subarray_handler(
    mut ctx: EnvironmentContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let cols = subarray_align_spec(&args[0])?;
    let mut res = array_from_opts(
        ctx.parser,
        ArrayOpts {
            hskip_before_and_after: Some(false),
            cols: Some(cols.clone()),
            array_stretch: Some(0.5),
            ..Default::default()
        },
        Style::Script,
    )?;
    if res.body.iter().any(|row| row.len() > 1) {
        return Err(ParseError::Expected);
    }
    Ok(ParseNode::Array(res))
}

fn cases_handler(
    mut ctx: EnvironmentContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let res = array_from_opts(
        ctx.parser,
        ArrayOpts {
            array_stretch: Some(1.2),
            cols: Some(vec![
                AlignSpec::Align {
                    align: Cow::Borrowed("l"),
                    pre_gap: Some(0.0),
                    post_gap: Some(1.0),
                },
                AlignSpec::Align {
                    align: Cow::Borrowed("l"),
                    pre_gap: Some(0.0),
                    post_gap: Some(0.0),
                },
            ]),
            ..Default::default()
        },
        d_cell_style(&ctx.env_name),
    )?;

    let (left, right) = if ctx.env_name.contains('r') {
        (".".to_string(), "\\}".to_string())
    } else {
        ("\\{".to_string(), ".".to_string())
    };

    Ok(ParseNode::LeftRight(LeftRightNode {
        left,
        right,
        right_color: None,
        body: vec![ParseNode::Array(res)],
        info: NodeInfo::new_mode(ctx.mode),
    }))
}

fn gathered_handler(
    mut ctx: EnvironmentContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    if ctx.env_name == "gather" || ctx.env_name == "gather*" {
        validate_display(&ctx)?;
    }
    let res = array_from_opts(
        ctx.parser,
        ArrayOpts {
            cols: Some(vec![AlignSpec::Align {
                align: Cow::Borrowed("c"),
                pre_gap: None,
                post_gap: None,
            }]),
            add_jot: Some(true),
            col_separation_type: Some(ColSeparationType::Gather),
            empty_single_row: true,
            leq_no: Some(ctx.parser.conf.leq_no),
            auto_tag: if ctx.env_name.contains("ed") {
                None
            } else {
                Some(!ctx.env_name.contains('*'))
            },
            ..Default::default()
        },
        Style::Display,
    )?;
    Ok(ParseNode::Array(res))
}

fn equation_handler(
    mut ctx: EnvironmentContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    validate_display(&ctx)?;
    let res = array_from_opts(
        ctx.parser,
        ArrayOpts {
            empty_single_row: true,
            single_row: true,
            max_num_cols: Some(1),
            leq_no: Some(ctx.parser.conf.leq_no),
            auto_tag: Some(!ctx.env_name.contains('*')),
            ..Default::default()
        },
        Style::Display,
    )?;
    Ok(ParseNode::Array(res))
}

fn is_start_of_arrow(node: &ParseNode) -> bool {
    node.text() == Some("@")
}

fn is_label_end(node: &ParseNode, end: char) -> bool {
    node.text().map(|t| t == end.to_string()).unwrap_or(false)
}

fn cd_arrow_char(node: &ParseNode) -> Result<char, ParseError> {
    let text = node.text().ok_or(ParseError::Expected)?;
    text.chars().next().ok_or(ParseError::Expected)
}

fn cd_placeholder_arrow(
    arrow_char: char,
    labels: &[OrdGroupNode; 2],
    mode: Mode,
) -> ParseNode {
    let mut body = Vec::new();
    if !labels[0].body.is_empty() {
        body.push(ParseNode::OrdGroup(labels[0].clone()));
    }
    body.push(ParseNode::MathOrd(crate::parse_node::MathOrdNode {
        text: arrow_char.to_string(),
        info: NodeInfo::new_mode(mode),
    }));
    if !labels[1].body.is_empty() {
        body.push(ParseNode::OrdGroup(labels[1].clone()));
    }

    wrap_cell(body, Style::Display, mode)
}

fn normalize_arrow_char(c: char) -> char {
    match c {
        '\u{2223}' => '|', // Vertical bar parsed as U+2223 in the symbol table
        _ => c,
    }
}

fn parse_cd(parser: &mut Parser) -> Result<ArrayNode, ParseError> {
    let dbg_enabled = false;
    let mut parsed_rows: Vec<Vec<ParseNode>> = Vec::new();
    parser.gullet.begin_group();
    parser
        .gullet
        .macros
        .set_back_macro("\\cr", Some(Arc::new(MacroReplace::Text("\\\\\\relax".to_string()))));
    parser.gullet.begin_group();

    loop {
        parsed_rows.push(parser.dispatch_parse_expression(false, Some(BreakToken::DoubleBackslash))?);
        parser.gullet.end_group();
        parser.gullet.begin_group();
        let next = parser.fetch()?.content.clone();
        if next == "&" || next == "\\\\" {
            parser.consume();
        } else if next == "\\end" {
            if parsed_rows.last().map(|r| r.is_empty()).unwrap_or(false) {
                parsed_rows.pop();
            }
            break;
        } else {
            if dbg_enabled {
                eprintln!("cd parse: expected & or \\\\, got {}", next);
            }
            return Err(ParseError::Expected);
        }
    }

    let mut body: Vec<Vec<ParseNode>> = Vec::new();
    let mut row = Vec::new();
    for (i, row_nodes) in parsed_rows.into_iter().enumerate() {
        let mut cell = wrap_cell(Vec::new(), Style::Display, parser.mode());
        let mut j = 0;
        while j < row_nodes.len() {
            if !is_start_of_arrow(&row_nodes[j]) {
                if let ParseNode::Styling(sty) = &mut cell {
                    sty.body.push(row_nodes[j].clone());
                }
                j += 1;
                continue;
            }

            row.push(cell);
            j += 1;
            if j >= row_nodes.len() {
                if dbg_enabled {
                    eprintln!("cd parse: arrow at end of row");
                }
                return Err(ParseError::Expected);
            }
            let arrow_char = normalize_arrow_char(cd_arrow_char(&row_nodes[j])?);
            let mut labels = [
                OrdGroupNode {
                    body: Vec::new(),
                    semi_simple: None,
                    info: NodeInfo::new_mode(Mode::Math),
                },
                OrdGroupNode {
                    body: Vec::new(),
                    semi_simple: None,
                    info: NodeInfo::new_mode(Mode::Math),
                },
            ];

            if "=|.".contains(arrow_char) {
                // no labels
            } else if "<>AV".contains(arrow_char) {
                for label_idx in 0..2 {
                    let mut found = false;
                    while j + 1 < row_nodes.len() {
                        j += 1;
                        if is_label_end(&row_nodes[j], arrow_char) {
                            found = true;
                            break;
                        }
                        if is_start_of_arrow(&row_nodes[j]) {
                            if dbg_enabled {
                                eprintln!("cd parse: nested arrow found parsing labels");
                            }
                            return Err(ParseError::Expected);
                        }
                        labels[label_idx].body.push(row_nodes[j].clone());
                    }
                    if !found {
                        if dbg_enabled {
                            eprintln!("cd parse: missing closing {}", arrow_char);
                        }
                        return Err(ParseError::Expected);
                    }
                }
            } else {
                if dbg_enabled {
                    eprintln!("cd parse: unknown arrow char {}", arrow_char);
                }
                return Err(ParseError::Expected);
            }

            let arrow = cd_placeholder_arrow(arrow_char, &labels, parser.mode());
            row.push(arrow);
            cell = wrap_cell(Vec::new(), Style::Display, parser.mode());
            j += 1;
        }

        if i % 2 == 0 {
            row.push(cell);
        } else if !row.is_empty() {
            row.remove(0);
        }

        body.push(std::mem::take(&mut row));
    }

    parser.gullet.end_group();
    parser.gullet.end_group();

    let cols_len = body.first().map(|r| r.len()).unwrap_or(0);
    let row_count = body.len();
    let cols = Some(
        (0..cols_len)
            .map(|_| AlignSpec::Align {
                align: Cow::Borrowed("c"),
                pre_gap: Some(0.25),
                post_gap: Some(0.25),
            })
            .collect(),
    );

    Ok(ArrayNode {
        body,
        col_separation_type: Some(ColSeparationType::Cd),
        h_skip_before_and_after: None,
        add_jot: Some(true),
        cols,
        array_stretch: 1.0,
        row_gaps: vec![None],
        h_lines_before_row: vec![Vec::new(); row_count + 1],
        tags: None,
        leq_no: None,
        is_cd: Some(true),
        info: NodeInfo::new_mode(Mode::Math),
    })
}

pub fn add_environments(envs: &mut Environments) {
    let push = |names: &[&'static str],
                prop: FunctionPropSpec,
                handler: Box<
                    dyn Fn(EnvironmentContext, &[ParseNode], &[Option<ParseNode>])
                        -> Result<ParseNode, ParseError>
                        + Send
                        + Sync,
                >,
                envs: &mut Environments| {
        let spec = Arc::new(EnvironmentSpec { prop, handler });
        for name in names {
            envs.insert(*name, spec.clone());
        }
    };

    push(
        &["array", "darray"],
        FunctionPropSpec::new_num_args(crate::parse_node::ParseNodeType::Array, 1)
            .with_allowed_in_text(true),
        Box::new(array_handler),
        envs,
    );

    push(
        &[
            "matrix",
            "pmatrix",
            "bmatrix",
            "Bmatrix",
            "vmatrix",
            "Vmatrix",
            "matrix*",
            "pmatrix*",
            "bmatrix*",
            "Bmatrix*",
            "vmatrix*",
            "Vmatrix*",
        ],
        FunctionPropSpec::new_num_args(crate::parse_node::ParseNodeType::Array, 0)
            .with_allowed_in_text(true),
        Box::new(matrix_handler),
        envs,
    );

    push(
        &["smallmatrix"],
        FunctionPropSpec::new_num_args(crate::parse_node::ParseNodeType::Array, 0)
            .with_allowed_in_text(true),
        Box::new(smallmatrix_handler),
        envs,
    );

    push(
        &["subarray"],
        FunctionPropSpec::new_num_args(crate::parse_node::ParseNodeType::Array, 1)
            .with_allowed_in_text(true),
        Box::new(subarray_handler),
        envs,
    );

    push(
        &["cases", "dcases", "rcases", "drcases"],
        FunctionPropSpec::new_num_args(crate::parse_node::ParseNodeType::Array, 0)
            .with_allowed_in_text(true),
        Box::new(cases_handler),
        envs,
    );

    push(
        &["align", "align*", "aligned", "split"],
        FunctionPropSpec::new_num_args(crate::parse_node::ParseNodeType::Array, 0)
            .with_allowed_in_text(true),
        Box::new(aligned_handler),
        envs,
    );

    push(
        &["alignat", "alignat*", "alignedat"],
        FunctionPropSpec::new_num_args(crate::parse_node::ParseNodeType::Array, 1)
            .with_allowed_in_text(true),
        Box::new(aligned_handler),
        envs,
    );

    push(
        &["gathered", "gather", "gather*"],
        FunctionPropSpec::new_num_args(crate::parse_node::ParseNodeType::Array, 0)
            .with_allowed_in_text(true),
        Box::new(gathered_handler),
        envs,
    );

    push(
        &["equation", "equation*"],
        FunctionPropSpec::new_num_args(crate::parse_node::ParseNodeType::Array, 0)
            .with_allowed_in_text(true),
        Box::new(equation_handler),
        envs,
    );

    push(
        &["CD"],
        FunctionPropSpec::new_num_args(crate::parse_node::ParseNodeType::Array, 0)
            .with_allowed_in_text(true),
        Box::new(|mut ctx, _, _| {
            validate_display(&ctx)?;
            let res = parse_cd(ctx.parser)?;
            Ok(ParseNode::Array(res))
        }),
        envs,
    );
}
