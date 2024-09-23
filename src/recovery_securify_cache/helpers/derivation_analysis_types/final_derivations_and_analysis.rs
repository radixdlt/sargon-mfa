use crate::prelude::*;

/// The final outcome of `PolyDerivation::poly_derive`, used
/// by operations such as Account Recovery Scan and
/// Securifying accounts
#[derive(Clone, Debug)]
pub struct FinalDerivationsAndAnalysis {
    // OR `KnownTakenInstances` ??
    pub derived_instances: DerivedFactorInstances,
    pub cache: Arc<PreDerivedKeysCache>,
}
