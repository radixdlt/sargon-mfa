use crate::prelude::*;

/// A request to derive keys using a single factor source, dispatched by the
/// interactor to the host, request created by the KeysCollector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonoFactorKeyDerivationRequest {
    /// ID of the FactorSource used to derive keys.
    pub(crate) factor_source_id: FactorSourceIDFromHash,

    /// The derivation paths used to derive keys using
    /// the factor source
    pub(crate) derivation_paths: IndexSet<DerivationPath>,
}

impl MonoFactorKeyDerivationRequest {
    pub(crate) fn new(
        factor_source_id: FactorSourceIDFromHash,
        derivation_paths: IndexSet<DerivationPath>,
    ) -> Self {
        Self {
            factor_source_id,
            derivation_paths,
        }
    }
}
