use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum AlignSpec {
    Separator(Cow<'static, str>),
    Align {
        align: Cow<'static, str>,
        // TODO: are these units
        pre_gap: Option<usize>,
        post_gap: Option<usize>,
    },
}

/// Indicate column separation in MathML
#[derive(Debug, Clone)]
pub enum ColSeparationType {
    Align,
    AlignAt,
    Gather,
    Small,
    Cd,
}
