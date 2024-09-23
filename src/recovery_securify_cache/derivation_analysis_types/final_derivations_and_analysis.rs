use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct FinalDerivationsAndAnalysis {
    // OR `KnownTakenInstances` ??
    pub derived_instances: DerivedFactorInstances,
    pub cache: Arc<PreDerivedKeysCache>,
}
