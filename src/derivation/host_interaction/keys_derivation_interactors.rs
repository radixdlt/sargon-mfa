use crate::prelude::*;

/// A collection of "interactors" which can derive keys.
pub trait KeysDerivationInteractors {
    fn interactor_for(&self, kind: FactorSourceKind) -> KeyDerivationInteractor;
}
