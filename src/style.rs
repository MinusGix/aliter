#[repr(u8)]
pub enum StyleId {
    D = 0,
    Dc = 1,
    T = 2,
    Tc = 3,
    S = 4,
    Sc = 5,
    SS = 6,
    SSc = 7,
}
impl StyleId {
    // TODO: is this a float
    pub fn size(&self) -> usize {
        match self {
            StyleId::D => 0,
            StyleId::Dc => 0,
            StyleId::T => 1,
            StyleId::Tc => 1,
            StyleId::S => 2,
            StyleId::Sc => 2,
            StyleId::SS => 3,
            StyleId::SSc => 3,
        }
    }

    pub fn cramped(&self) -> bool {
        match self {
            StyleId::D => false,
            StyleId::Dc => true,
            StyleId::T => false,
            StyleId::Tc => true,
            StyleId::S => false,
            StyleId::Sc => true,
            StyleId::SS => false,
            StyleId::SSc => true,
        }
    }

    pub fn tight(&self) -> bool {
        self.size() >= 2
    }
}

pub(crate) const SUP: &[StyleId] = &[
    StyleId::S,
    StyleId::Sc,
    StyleId::S,
    StyleId::Sc,
    StyleId::SS,
    StyleId::SSc,
    StyleId::SS,
    StyleId::SSc,
];
pub(crate) const SUB: &[StyleId] = &[
    StyleId::Sc,
    StyleId::Sc,
    StyleId::Sc,
    StyleId::Sc,
    StyleId::SSc,
    StyleId::SSc,
    StyleId::SSc,
    StyleId::SSc,
];
pub(crate) const FRACNUM: &[StyleId] = &[
    StyleId::T,
    StyleId::Tc,
    StyleId::S,
    StyleId::Sc,
    StyleId::SS,
    StyleId::SSc,
    StyleId::SS,
    StyleId::SSc,
];
pub(crate) const FRACDEN: &[StyleId] = &[
    StyleId::Tc,
    StyleId::Tc,
    StyleId::Sc,
    StyleId::Sc,
    StyleId::SSc,
    StyleId::SSc,
    StyleId::SSc,
    StyleId::SSc,
];
pub(crate) const CRAMP: &[StyleId] = &[
    StyleId::Dc,
    StyleId::Dc,
    StyleId::Tc,
    StyleId::Tc,
    StyleId::Sc,
    StyleId::Sc,
    StyleId::SSc,
    StyleId::SSc,
];
pub(crate) const TEXT: &[StyleId] = &[
    StyleId::D,
    StyleId::Dc,
    StyleId::T,
    StyleId::Tc,
    StyleId::T,
    StyleId::Tc,
    StyleId::T,
    StyleId::Tc,
];

pub(crate) const DISPLAY_STYLE: StyleId = StyleId::D;
pub(crate) const TEXT_STYLE: StyleId = StyleId::T;
pub(crate) const SCRIPT_STYLE: StyleId = StyleId::S;
pub(crate) const SCRIPT_SCRIPT_STYLE: StyleId = StyleId::SS;
