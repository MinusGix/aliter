use crate::{
    expander::Mode,
    font_metrics_data,
    unicode_scripts::supported_codepoint,
    util::{char_code_for, find_assoc_data},
};

// TODO: is there a smarter way to take values from the table and initialize from them than using this enum? That is also easy to update when needed!
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetricName {
    Slant,
    Space,
    Stretch,
    Shrink,
    XHeight,
    Quad,
    ExtraSpace,
    Num1,
    Num2,
    Num3,
    Denom1,
    Denom2,
    Sup1,
    Sup2,
    Sup3,
    Sub1,
    Sub2,
    SupDrop,
    SubDrop,
    Delim1,
    Delim2,
    AxisHeight,

    DefaultRuleThickness,
    BigOpSpacing1,
    BigOpSpacing2,
    BigOpSpacing3,
    BigOpSpacing4,
    BigOpSpacing5,

    SqrtRuleThickness,

    PtPerEm,

    DoubleRuleSep,

    ArrayRuleWidth,

    FboxSep,
    FboxRule,

    CssEmPerMu,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FontMetrics {
    pub slant: f64,
    pub space: f64,
    pub stretch: f64,
    pub shrink: f64,
    pub x_height: f64,
    pub quad: f64,
    pub extra_space: f64,
    pub num1: f64,
    pub num2: f64,
    pub num3: f64,
    pub denom1: f64,
    pub denom2: f64,
    pub sup1: f64,
    pub sup2: f64,
    pub sup3: f64,
    pub sub1: f64,
    pub sub2: f64,
    pub sup_drop: f64,
    pub sub_drop: f64,
    pub delim1: f64,
    pub delim2: f64,
    pub axis_height: f64,

    pub default_rule_thickness: f64,
    pub big_op_spacing1: f64,
    pub big_op_spacing2: f64,
    pub big_op_spacing3: f64,
    pub big_op_spacing4: f64,
    pub big_op_spacing5: f64,

    pub sqrt_rule_thickness: f64,

    pub pt_per_em: f64,

    pub double_rule_sep: f64,

    pub array_rule_width: f64,

    pub fboxsep: f64,
    pub fboxrule: f64,

    pub css_em_per_mu: f64,
}
impl FontMetrics {
    fn update_from_name(&mut self, name: MetricName, v: f64) {
        match name {
            MetricName::Slant => self.slant = v,
            MetricName::Space => self.space = v,
            MetricName::Stretch => self.stretch = v,
            MetricName::Shrink => self.shrink = v,
            MetricName::XHeight => self.x_height = v,
            MetricName::Quad => self.quad = v,
            MetricName::ExtraSpace => self.extra_space = v,
            MetricName::Num1 => self.num1 = v,
            MetricName::Num2 => self.num2 = v,
            MetricName::Num3 => self.num3 = v,
            MetricName::Denom1 => self.denom1 = v,
            MetricName::Denom2 => self.denom2 = v,
            MetricName::Sup1 => self.sup1 = v,
            MetricName::Sup2 => self.sup2 = v,
            MetricName::Sup3 => self.sup3 = v,
            MetricName::Sub1 => self.sub1 = v,
            MetricName::Sub2 => self.sub2 = v,
            MetricName::SupDrop => self.sup_drop = v,
            MetricName::SubDrop => self.sub_drop = v,
            MetricName::Delim1 => self.delim1 = v,
            MetricName::Delim2 => self.delim2 = v,
            MetricName::AxisHeight => self.axis_height = v,

            MetricName::DefaultRuleThickness => self.default_rule_thickness = v,
            MetricName::BigOpSpacing1 => self.big_op_spacing1 = v,
            MetricName::BigOpSpacing2 => self.big_op_spacing2 = v,
            MetricName::BigOpSpacing3 => self.big_op_spacing3 = v,
            MetricName::BigOpSpacing4 => self.big_op_spacing4 = v,
            MetricName::BigOpSpacing5 => self.big_op_spacing5 = v,

            MetricName::SqrtRuleThickness => self.sqrt_rule_thickness = v,

            MetricName::PtPerEm => self.pt_per_em = v,

            MetricName::DoubleRuleSep => self.double_rule_sep = v,

            MetricName::ArrayRuleWidth => self.array_rule_width = v,

            MetricName::FboxSep => self.fboxsep = v,
            MetricName::FboxRule => self.fboxrule = v,

            MetricName::CssEmPerMu => self.css_em_per_mu = v,
        }
    }
}

const SIGMAS_AND_XIS: &'static [(MetricName, [f64; 3])] = &[
    (MetricName::Slant, [0.250, 0.250, 0.250]),      // sigma1
    (MetricName::Space, [0.000, 0.000, 0.000]),      // sigma2
    (MetricName::Stretch, [0.000, 0.000, 0.000]),    // sigma3
    (MetricName::Shrink, [0.000, 0.000, 0.000]),     // sigma4
    (MetricName::XHeight, [0.431, 0.431, 0.431]),    // sigma5
    (MetricName::Quad, [1.000, 1.171, 1.472]),       // sigma6
    (MetricName::ExtraSpace, [0.000, 0.000, 0.000]), // sigma7
    (MetricName::Num1, [0.677, 0.732, 0.925]),       // sigma8
    (MetricName::Num2, [0.394, 0.384, 0.387]),       // sigma9
    (MetricName::Num3, [0.444, 0.471, 0.504]),       // sigma10
    (MetricName::Denom1, [0.686, 0.752, 1.025]),     // sigma11
    (MetricName::Denom2, [0.345, 0.344, 0.532]),     // sigma12
    (MetricName::Sup1, [0.413, 0.503, 0.504]),       // sigma13
    (MetricName::Sup2, [0.363, 0.431, 0.404]),       // sigma14
    (MetricName::Sup3, [0.289, 0.286, 0.294]),       // sigma15
    (MetricName::Sub1, [0.150, 0.143, 0.200]),       // sigma16
    (MetricName::Sub2, [0.247, 0.286, 0.400]),       // sigma17
    (MetricName::SupDrop, [0.386, 0.353, 0.494]),    // sigma18
    (MetricName::SubDrop, [0.050, 0.071, 0.100]),    // sigma19
    (MetricName::Delim1, [2.390, 1.700, 1.980]),     // sigma20
    (MetricName::Delim2, [1.010, 1.157, 1.420]),     // sigma21
    (MetricName::AxisHeight, [0.250, 0.250, 0.250]), // sigma22
    // These font metrics are extracted from TeX by using tftopl on cmex10.tfm;
    // they correspond to the font parameters of the extension fonts (family 3).
    // See the TeXbook, page 441. In AMSTeX, the extension fonts scale; to
    // match cmex7, we'd use cmex7.tfm values for script and scriptscript
    // values.
    (MetricName::DefaultRuleThickness, [0.04, 0.049, 0.049]), // xi8; cmex7: 0.049
    (MetricName::BigOpSpacing1, [0.111, 0.111, 0.111]),       // xi9
    (MetricName::BigOpSpacing2, [0.166, 0.166, 0.166]),       // xi10
    (MetricName::BigOpSpacing3, [0.2, 0.2, 0.2]),             // xi11
    (MetricName::BigOpSpacing4, [0.6, 0.611, 0.611]),         // xi12; cmex7: 0.611
    (MetricName::BigOpSpacing5, [0.1, 0.143, 0.143]),         // xi13; cmex7: 0.143
    // The \sqrt rule width is taken from the height of the surd character.
    // Since we use the same font at all sizes, this thickness doesn't scale.
    (MetricName::SqrtRuleThickness, [0.04, 0.04, 0.04]), // xi15
    // This value determines how large a pt is, for metrics which are defined
    // in terms of pts.
    // This value is also used in katex.less; if you change it make sure the
    // values match.
    (MetricName::PtPerEm, [10.0, 10.0, 10.0]),
    // The space between adjacent `|` columns in an array definition. From
    // `\showthe\doublerulesep` in LaTeX. Equals 2.0 / ptPerEm.
    (MetricName::DoubleRuleSep, [0.2, 0.2, 0.2]),
    // The width of separator lines in {array} environments. From
    // `\showthe\arrayrulewidth` in LaTeX. Equals 0.4 / ptPerEm.
    (MetricName::ArrayRuleWidth, [0.04, 0.04, 0.04]),
    // Two values from LaTeX source2e:
    (MetricName::FboxSep, [0.3, 0.3, 0.3]), //        3 pt / ptPerEm
    (MetricName::FboxRule, [0.04, 0.04, 0.04]), // 0.4 pt / ptPerEm
];

const EXTRA_CHARACTER_MAP: &[(char, char)] = &[
    // Latin-1
    ('Å', 'A'),
    ('Ð', 'D'),
    ('Þ', 'o'),
    ('å', 'a'),
    ('ð', 'd'),
    ('þ', 'o'),
    // Cyrillic
    ('А', 'A'),
    ('Б', 'B'),
    ('В', 'B'),
    ('Г', 'F'),
    ('Д', 'A'),
    ('Е', 'E'),
    ('Ж', 'K'),
    ('З', '3'),
    ('И', 'N'),
    ('Й', 'N'),
    ('К', 'K'),
    ('Л', 'N'),
    ('М', 'M'),
    ('Н', 'H'),
    ('О', 'O'),
    ('П', 'N'),
    ('Р', 'P'),
    ('С', 'C'),
    ('Т', 'T'),
    ('У', 'y'),
    ('Ф', 'O'),
    ('Х', 'X'),
    ('Ц', 'U'),
    ('Ч', 'h'),
    ('Ш', 'W'),
    ('Щ', 'W'),
    ('Ъ', 'B'),
    ('Ы', 'X'),
    ('Ь', 'B'),
    ('Э', '3'),
    ('Ю', 'X'),
    ('Я', 'R'),
    ('а', 'a'),
    ('б', 'b'),
    ('в', 'a'),
    ('г', 'r'),
    ('д', 'y'),
    ('е', 'e'),
    ('ж', 'm'),
    ('з', 'e'),
    ('и', 'n'),
    ('й', 'n'),
    ('к', 'n'),
    ('л', 'n'),
    ('м', 'm'),
    ('н', 'n'),
    ('о', 'o'),
    ('п', 'n'),
    ('р', 'p'),
    ('с', 'c'),
    ('т', 'o'),
    ('у', 'y'),
    ('ф', 'b'),
    ('х', 'x'),
    ('ц', 'n'),
    ('ч', 'n'),
    ('ш', 'w'),
    ('щ', 'w'),
    ('ъ', 'a'),
    ('ы', 'm'),
    ('ь', 'a'),
    ('э', 'e'),
    ('ю', 'm'),
    ('я', 'r'),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CharacterMetrics {
    pub depth: f64,
    pub height: f64,
    pub italic: f64,
    pub skew: f64,
    pub width: f64,
}
impl CharacterMetrics {
    fn from_metric(metric: [f64; 5]) -> Self {
        Self {
            depth: metric[0],
            height: metric[1],
            italic: metric[2],
            skew: metric[3],
            width: metric[4],
        }
    }
}

// TODO: set_font_metrics

/// This function is a convenience function for lookuping up information in the [`METRIC_MAP`]
/// table.  
/// Note that you typical callers in KaTeX only pass in the first character of the string.
pub(crate) fn get_character_metrics(
    character: char,
    font: &str,
    mode: Mode,
) -> Option<CharacterMetrics> {
    let metrics = if let Some(metrics) = font_metrics_data::get_metric(font) {
        metrics
    } else {
        panic!("Font metrics not found for font: {font:?}");
    };

    let ch = char_code_for(character);

    // TODO: can we flatten this if/else nesting?
    if let Some(metric) = find_assoc_data(metrics, ch) {
        Some(CharacterMetrics::from_metric(*metric))
    } else {
        let ch = find_assoc_data(EXTRA_CHARACTER_MAP, character);
        if let Some(ch) = ch {
            let ch = char_code_for(*ch);
            if let Some(metric) = find_assoc_data(metrics, ch) {
                Some(CharacterMetrics::from_metric(*metric))
            } else if mode == Mode::Text {
                if supported_codepoint(ch) {
                    // We default to using 'M' for characters we don't have metrics for.
                    let ch = char_code_for('M'); // 77
                    if let Some(metric) = find_assoc_data(metrics, ch) {
                        Some(CharacterMetrics::from_metric(*metric))
                    } else {
                        // TODO: warn?
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSizeIndex {
    Zero,
    One,
    Two,
}
impl FontSizeIndex {
    pub fn as_usize(self) -> usize {
        match self {
            Self::Zero => 0,
            Self::One => 1,
            Self::Two => 2,
        }
    }
}

// TODO: We don't do a cache like katex does. Is that actually needed? How often is this called?
fn font_metrics_by_size_index(size_index: FontSizeIndex) -> FontMetrics {
    // TODO: make sure this gets optimized out?
    let css_em_per_mu = find_assoc_data(SIGMAS_AND_XIS, MetricName::Quad).unwrap();
    let css_em_per_mu = css_em_per_mu[size_index.as_usize()];
    let css_em_per_mu = css_em_per_mu / 18.0;

    let mut font_metrics = FontMetrics::default();

    font_metrics.update_from_name(MetricName::CssEmPerMu, css_em_per_mu);

    for (name, v) in SIGMAS_AND_XIS {
        let v = v[size_index.as_usize()];
        font_metrics.update_from_name(*name, v);
    }

    font_metrics
}

pub(crate) fn get_global_metrics(size: usize) -> FontMetrics {
    let size_index = if size >= 5 {
        FontSizeIndex::Zero
    } else if size >= 3 {
        FontSizeIndex::One
    } else {
        FontSizeIndex::Two
    };

    font_metrics_by_size_index(size_index)
}
