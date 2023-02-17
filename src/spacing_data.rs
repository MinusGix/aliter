//! Describes spaces between different classes of atoms.

use crate::{html::DomType, unit::Mu};

const THIN_SPACE: Mu = Mu(3.0);
const MEDIUM_SPACE: Mu = Mu(4.0);
const THICK_SPACE: Mu = Mu(5.0);

/// Spacing relationships for display and text styles
pub(crate) const SPACINGS: &'static [((DomType, DomType), Mu)] = &[
    ((DomType::MOrd, DomType::MOp), THIN_SPACE),
    ((DomType::MOrd, DomType::MBin), MEDIUM_SPACE),
    ((DomType::MOrd, DomType::MRel), THICK_SPACE),
    ((DomType::MOrd, DomType::MInner), THIN_SPACE),
    //
    ((DomType::MOp, DomType::MOrd), THIN_SPACE),
    ((DomType::MOp, DomType::MOp), THIN_SPACE),
    ((DomType::MOp, DomType::MRel), THICK_SPACE),
    ((DomType::MOp, DomType::MInner), THIN_SPACE),
    //
    ((DomType::MBin, DomType::MOrd), MEDIUM_SPACE),
    ((DomType::MBin, DomType::MOp), MEDIUM_SPACE),
    ((DomType::MBin, DomType::MOpen), MEDIUM_SPACE),
    ((DomType::MBin, DomType::MInner), MEDIUM_SPACE),
    //
    ((DomType::MRel, DomType::MOrd), THICK_SPACE),
    ((DomType::MRel, DomType::MOp), THICK_SPACE),
    ((DomType::MRel, DomType::MOpen), THICK_SPACE),
    ((DomType::MRel, DomType::MInner), THICK_SPACE),
    //
    ((DomType::MClose, DomType::MOp), THIN_SPACE),
    ((DomType::MClose, DomType::MBin), MEDIUM_SPACE),
    ((DomType::MClose, DomType::MRel), THICK_SPACE),
    ((DomType::MClose, DomType::MInner), THIN_SPACE),
    //
    ((DomType::MPunct, DomType::MOrd), THIN_SPACE),
    ((DomType::MPunct, DomType::MOp), THIN_SPACE),
    ((DomType::MPunct, DomType::MRel), THICK_SPACE),
    ((DomType::MPunct, DomType::MOpen), THIN_SPACE),
    ((DomType::MPunct, DomType::MClose), THIN_SPACE),
    ((DomType::MPunct, DomType::MPunct), THIN_SPACE),
    ((DomType::MPunct, DomType::MInner), THIN_SPACE),
    //
    ((DomType::MInner, DomType::MOrd), THIN_SPACE),
    ((DomType::MInner, DomType::MOp), THIN_SPACE),
    ((DomType::MInner, DomType::MBin), MEDIUM_SPACE),
    ((DomType::MInner, DomType::MRel), THICK_SPACE),
    ((DomType::MInner, DomType::MOpen), THIN_SPACE),
    ((DomType::MInner, DomType::MPunct), THIN_SPACE),
    ((DomType::MInner, DomType::MInner), THIN_SPACE),
];

/// Spacing relationships for script and scriptscript styles
pub(crate) const TIGHT_SPACINGS: &'static [((DomType, DomType), Mu)] = &[
    ((DomType::MOrd, DomType::MOp), THIN_SPACE),
    //
    ((DomType::MOp, DomType::MOrd), THIN_SPACE),
    ((DomType::MOp, DomType::MOp), THIN_SPACE),
    //
    ((DomType::MClose, DomType::MOp), THIN_SPACE),
    //
    ((DomType::MInner, DomType::MOp), THIN_SPACE),
];
