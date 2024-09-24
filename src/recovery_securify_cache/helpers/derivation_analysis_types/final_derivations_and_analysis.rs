use crate::prelude::*;

/// The final outcome of `PolyDerivation::poly_derive`, used
/// by operations such as Account Recovery Scan and
/// Securifying accounts
#[derive(Clone, Debug)]
pub struct FinalDerivationsAndAnalysis {
    pub entities_from_analysis: EntitiesFromAnalysis,
    pub cache: Arc<PreDerivedKeysCache>,
}
