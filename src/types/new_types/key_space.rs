use crate::prelude::*;

/// We split the hardened derivation entity index "space" in
/// two halves. The first half is used for unsecurified entities,
/// and the second half is used for securified entities.
///
/// The Unsecurified half works as it does today, with hardened
/// `u32` values, where hardened denotes addition of `2^31`.
///
/// The Securified half is a new concept, where we offset the
/// `u32` value with half of the 2^31 space, i.e. `2^30`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, derive_more::Display, derive_more::Debug)]
pub enum KeySpace {
    /// Used by FactorInstances controlling
    /// unsecurified entities, called "VECI"s
    /// Virtual Entity Creating (Factor)Instances.
    #[display("Unsecurified")]
    #[debug("Unsecurified")]
    Unsecurified,

    /// Used by FactorInstances in MatrixOfFactorInstances
    /// for securified entities.
    ///
    /// This is the entity base index value, `u32` `+ 2^30`.
    ///
    /// We use `6^` notation to indicate: `6' + 2^30`, where `'`,
    /// is the standard notation for hardened indices.
    #[display("Securified")]
    #[debug("Securified")]
    Securified,
}

impl KeySpace {
    pub fn both() -> [Self; 2] {
        [Self::Unsecurified, Self::Securified]
    }

    pub fn indicator(&self) -> String {
        match self {
            Self::Unsecurified => "'".to_owned(),
            Self::Securified => "^".to_owned(),
        }
    }
}
