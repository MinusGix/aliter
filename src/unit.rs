use crate::Options;

/// Round `n` to 4 decimal places, or to the nearest 1/10,000th em.
pub(crate) fn make_em(n: f64) -> String {
    format!("{:.4}em", n)
}

/// Convert a size measurement into a CSS em value for the current style/scale
pub(crate) fn calculate_size(size_val: &Measurement, options: &Options) -> f64 {
    let font_metrics = options.font_metrics();
    let scale = if let Some(pt_per_unit) = size_val.unit_pt_size() {
        // Convert pt to css em
        let css_em = pt_per_unit / font_metrics.pt_per_em;
        // Unscale to make absolute units
        css_em / options.size_multiplier()
    } else if let Measurement::Mu(mu) = &size_val {
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

    (size_val.num() * scale).min(options.max_size as f64)
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

macro_rules! mk_pt_unit {
    ( $(#[$outer:meta])* $name:ident : $pt_size:expr) => {
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
        }
    };
}

mk_pt_unit!(
    /// TeX point
    Pt : 1.0
);

mk_pt_unit!(
    /// Millimeter
    Mm : 7227.0 / 2540.0
);

mk_pt_unit!(
    /// Centimeter
    Cm : 7227.0 / 254.0
);

mk_pt_unit!(
    /// Inch
    In : 72.27
);

mk_pt_unit!(
    /// Big PostScript points
    Bp : 803.0 / 800.0
);

mk_pt_unit!(
    /// Pica
    Pc : 12.0
);

mk_pt_unit!(
    /// Didot
    Dd : 1238.0 / 1157.0
);

mk_pt_unit!(
    /// Cicero (12 didot)
    Cc : 14856.0 / 1157.0
);

mk_pt_unit!(
    /// New didot
    Nd : 685.0 / 642.0
);

mk_pt_unit!(
    /// New Cicero (12 didot)
    Nc : 1370.0 / 107.0
);

mk_pt_unit!(
    /// Scaled point (TeX's internal smallest unit)
    Sp : 1.0 / 65536.0
);

mk_pt_unit!(
    /// Pixel
    /// \pdfpxdimen defaults to 1 bp in pdfTeX and LuaTeX
    Px : 803.0 / 800.0
);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ex(pub f64);

/// An f64 in em, which is a unit relative to the font size of the parent
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Em(pub f64);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Mu(pub f64);
