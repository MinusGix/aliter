use std::{borrow::Cow, char::REPLACEMENT_CHARACTER, sync::Arc};

use crate::parse_node::{NodeInfo, ParseNode, ParseNodeType, TextOrdNode};

use super::{FunctionContext, FunctionPropSpec, FunctionSpec, Functions};

pub fn add_functions(fns: &mut Functions) {
    let chr = Arc::new(FunctionSpec {
        prop: FunctionPropSpec::new_num_args(ParseNodeType::TextOrd, 1).with_allowed_in_text(true),
        handler: Box::new(chr_handler),
        // TODO:
        #[cfg(feature = "html")]
        html_builder: None,
        // TODO
        #[cfg(feature = "mathml")]
        mathml_builder: None,
    });

    fns.insert(Cow::Borrowed("\\@char"), chr);
}

fn chr_handler(
    ctx: FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> ParseNode {
    let arg = if let ParseNode::OrdGroup(ord_group) = &args[0] {
        ord_group
    } else {
        // TODO: just return an error
        panic!();
    };

    let group = &arg.body;
    let mut number = String::new();
    for part in group.iter() {
        if let ParseNode::TextOrd(text_ord) = part {
            number.push_str(&text_ord.text);
        } else {
            // TODO: just return an error
            panic!();
        }
    }

    // TODO: This could probably be special cased for one/two/three character results to avoid
    // most allocations?

    // TODO: just return an error
    let code = number.parse::<u32>().unwrap();
    // TODO: I'm pretty uncertain about the correctness of this
    let text = if code >= 0x10_FFFF {
        // TODO: Don't panic
        panic!()
    } else if code <= 0xFFFF {
        char::decode_utf16([code as u16])
            .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
            .collect::<String>()
    } else {
        // Astral code point; split it into surrogate halves
        let code = code - 0x1_0000;
        // TODO: Is this correct?
        char::decode_utf16([
            ((code >> 10) + 0xD800) as u16,
            ((code & 0x03FF) + 0xDC00) as u16,
        ])
        .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
        .collect::<String>()
    };

    ParseNode::TextOrd(TextOrdNode {
        text: Cow::Owned(text),
        info: NodeInfo::new_mode(ctx.parser.mode()),
    })
}
