use crate::prelude::*;

/// An interactor which can sign transactions - either in poly or mono.
pub enum SignInteractor {
    PolyFactor(Arc<dyn PolyFactorSignInteractor>),
    MonoFactor(Arc<dyn MonoFactorSignInteractor>),
}

impl SignInteractor {
    pub(crate) fn poly(interactor: Arc<dyn PolyFactorSignInteractor>) -> Self {
        Self::PolyFactor(interactor)
    }

    pub(crate) fn mono(interactor: Arc<dyn MonoFactorSignInteractor>) -> Self {
        Self::MonoFactor(interactor)
    }
}
