use crate::Options;

/// Round `n` to 4 decimal places, or to the nearest 1/10,000th em.
pub(crate) fn make_em(n: f64) -> String {
    // 0.0 -> 0em
    // 0.1 -> 0.1em
    // 0.01 -> 0.01em
    // 0.001 -> 0.001em
    // 0.0001 -> 0.0001em
    // 0.00001 -> 0em
    // 0.56789 -> 0.5679em

    let mut n = n;
    if n.abs() < 0.00001 {
        n = 0.0;
    } else {
        n = (n * 10000.0).round() / 10000.0;
    }

    format!("{}em", n)
}

/// Convert a size measurement into a CSS em value for the current style/scale
pub(crate) fn calculate_size(size_val: &Measurement, options: &Options) -> f64 {
    let font_metrics = options.font_metrics();
    let scale = if let Some(pt_per_unit) = size_val.unit_pt_size() {
        // Convert pt to css em
        let css_em = pt_per_unit / font_metrics.pt_per_em;
        // Unscale to make absolute units
        css_em / options.size_multiplier()
    } else if let Measurement::Mu(_mu) = &size_val {
        // 'mu' units scale with scriptstyle/scriptscriptstyle
        font_metrics.css_em_per_mu
    } else {
        // Other relative units always refer ot the textstyle font in the current size
        let unit_options = if options.style.is_tight() {
            options.having_style(options.style.text())
        } else {
            None
        };
        let unit_options = unit_options.as_ref().unwrap_or(options);
        let unit_font_metrics = unit_options.font_metrics();

        let scale = match size_val {
            Measurement::Ex(_) => unit_font_metrics.x_height,
            Measurement::Em(_) => unit_font_metrics.quad,
            // TODO: Don't panic
            _ => panic!("Invalid unit: '{:?}'", size_val),
        };

        if unit_options == options {
            scale
        } else {
            scale * unit_options.size_multiplier() / options.size_multiplier()
        }
    };

    (size_val.num() * scale).min(options.max_size.0)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Measurement {
    Pt(Pt),
    Mm(Mm),
    Cm(Cm),
    In(In),
    Bp(Bp),
    Pc(Pc),
    Dd(Dd),
    Cc(Cc),
    Nd(Nd),
    Nc(Nc),
    Sp(Sp),
    Px(Px),

    Ex(Ex),
    Em(Em),
    Mu(Mu),
}
impl Measurement {
    pub fn num(&self) -> f64 {
        match self {
            Measurement::Pt(x) => x.0,
            Measurement::Mm(x) => x.0,
            Measurement::Cm(x) => x.0,
            Measurement::In(x) => x.0,
            Measurement::Bp(x) => x.0,
            Measurement::Pc(x) => x.0,
            Measurement::Dd(x) => x.0,
            Measurement::Cc(x) => x.0,
            Measurement::Nd(x) => x.0,
            Measurement::Nc(x) => x.0,
            Measurement::Sp(x) => x.0,
            Measurement::Px(x) => x.0,
            Measurement::Ex(x) => x.0,
            Measurement::Em(x) => x.0,
            Measurement::Mu(x) => x.0,
        }
    }

    pub fn is_relative(&self) -> bool {
        matches!(self, Self::Ex(_) | Self::Em(_) | Self::Mu(_))
    }

    pub fn from_unit(v: f64, unit: &str) -> Option<Measurement> {
        Some(match unit {
            "pt" => Measurement::Pt(Pt(v)),
            "mm" => Measurement::Mm(Mm(v)),
            "cm" => Measurement::Cm(Cm(v)),
            "in" => Measurement::In(In(v)),
            "bp" => Measurement::Bp(Bp(v)),
            "pc" => Measurement::Pc(Pc(v)),
            "dd" => Measurement::Dd(Dd(v)),
            "cc" => Measurement::Cc(Cc(v)),
            "nd" => Measurement::Nd(Nd(v)),
            "nc" => Measurement::Nc(Nc(v)),
            "sp" => Measurement::Sp(Sp(v)),
            "px" => Measurement::Px(Px(v)),

            "ex" => Measurement::Ex(Ex(v)),
            "em" => Measurement::Em(Em(v)),
            "mu" => Measurement::Mu(Mu(v)),

            _ => return None,
        })
    }

    pub fn unit_pt_size(&self) -> Option<f64> {
        Some(match self {
            Measurement::Pt(_) => Pt::unit_pt_size(),
            Measurement::Mm(_) => Mm::unit_pt_size(),
            Measurement::Cm(_) => Cm::unit_pt_size(),
            Measurement::In(_) => In::unit_pt_size(),
            Measurement::Bp(_) => Bp::unit_pt_size(),
            Measurement::Pc(_) => Pc::unit_pt_size(),
            Measurement::Dd(_) => Dd::unit_pt_size(),
            Measurement::Cc(_) => Cc::unit_pt_size(),
            Measurement::Nd(_) => Nd::unit_pt_size(),
            Measurement::Nc(_) => Nc::unit_pt_size(),
            Measurement::Sp(_) => Sp::unit_pt_size(),
            Measurement::Px(_) => Px::unit_pt_size(),
            Measurement::Ex(_) | Measurement::Em(_) | Measurement::Mu(_) => return None,
        })
    }
}
impl ToString for Measurement {
    fn to_string(&self) -> String {
        match self {
            Measurement::Pt(x) => x.to_string(),
            Measurement::Mm(x) => x.to_string(),
            Measurement::Cm(x) => x.to_string(),
            Measurement::In(x) => x.to_string(),
            Measurement::Bp(x) => x.to_string(),
            Measurement::Pc(x) => x.to_string(),
            Measurement::Dd(x) => x.to_string(),
            Measurement::Cc(x) => x.to_string(),
            Measurement::Nd(x) => x.to_string(),
            Measurement::Nc(x) => x.to_string(),
            Measurement::Sp(x) => x.to_string(),
            Measurement::Px(x) => x.to_string(),
            Measurement::Ex(x) => x.to_string(),
            Measurement::Em(x) => x.to_string(),
            Measurement::Mu(x) => x.to_string(),
        }
    }
}

macro_rules! mk_pt_unit {
    ( $(#[$outer:meta])* $name:ident ($text:expr) : $pt_size:expr) => {
        $(#[$outer])*
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub struct $name(pub f64);
        impl $name {
            pub fn pt_size(&self) -> f64 {
                self.0 * Self::unit_pt_size()
            }

            pub fn unit_pt_size() -> f64 {
                $pt_size
            }

            pub fn name() -> &'static str {
                $text
            }
        }
        impl ToString for $name {
            fn to_string(&self) -> String {
                format!("{}{}", self.0, $text)
            }
        }
    };
}

mk_pt_unit!(
    /// TeX point
    Pt ("pt") : 1.0
);

mk_pt_unit!(
    /// Millimeter
    Mm ("mm") : 7227.0 / 2540.0
);

mk_pt_unit!(
    /// Centimeter
    Cm ("cm") : 7227.0 / 254.0
);

mk_pt_unit!(
    /// Inch
    In ("in") : 72.27
);

mk_pt_unit!(
    /// Big PostScript points
    Bp ("bp" ): 803.0 / 800.0
);

mk_pt_unit!(
    /// Pica
    Pc ("pc") : 12.0
);

mk_pt_unit!(
    /// Didot
    Dd ("dd") : 1238.0 / 1157.0
);

mk_pt_unit!(
    /// Cicero (12 didot)
    Cc ("cc") : 14856.0 / 1157.0
);

mk_pt_unit!(
    /// New didot
    Nd ("nd") : 685.0 / 642.0
);

mk_pt_unit!(
    /// New Cicero (12 didot)
    Nc ("nc") : 1370.0 / 107.0
);

mk_pt_unit!(
    /// Scaled point (TeX's internal smallest unit)
    Sp ("sp") : 1.0 / 65536.0
);

mk_pt_unit!(
    /// Pixel
    /// \pdfpxdimen defaults to 1 bp in pdfTeX and LuaTeX
    Px ("px") : 803.0 / 800.0
);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ex(pub f64);
impl Ex {
    pub fn name() -> &'static str {
        "ex"
    }
}
impl ToString for Ex {
    fn to_string(&self) -> String {
        format!("{}ex", self.0)
    }
}

/// An f64 in em, which is a unit relative to the font size of the parent
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Em(pub f64);
impl Em {
    pub fn name() -> &'static str {
        "em"
    }
}
// Note: you should typically use make_em() instead of this
impl ToString for Em {
    fn to_string(&self) -> String {
        format!("{}em", self.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Mu(pub f64);
impl Mu {
    pub fn name() -> &'static str {
        "mu"
    }
}
impl ToString for Mu {
    fn to_string(&self) -> String {
        format!("{}mu", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_em() {
        assert_eq!(make_em(0.0), "0em");
        assert_eq!(make_em(0.1), "0.1em");
        assert_eq!(make_em(0.01), "0.01em");
        assert_eq!(make_em(0.001), "0.001em");
        assert_eq!(make_em(0.0001), "0.0001em");
        assert_eq!(make_em(0.00001), "0em");
        assert_eq!(make_em(0.56789), "0.5679em");
        assert_eq!(make_em(1.42), "1.42em");
        assert_eq!(make_em(1.0), "1em");
    }
}
