use once_cell::sync::Lazy;

use crate::macr::Macros;

pub static BUILTIN_MACROS: Lazy<Macros> = Lazy::new(|| {
    let mut macros = Macros::default();

    macros
});
