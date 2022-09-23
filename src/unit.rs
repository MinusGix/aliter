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
