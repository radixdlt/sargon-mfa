#![allow(unused)]

use crate::prelude::*;

pub struct ProfileNextIndexAnalyzer {}

pub struct DeriveAndAnalyzeInput {
    factor_sources: IndexSet<HDFactorSource>,
    ids_of_new_factor_sources: IndexSet<FactorSourceIDFromHash>,
    cache: Option<PreDerivedKeysCache>,
    gateway: Arc<dyn Gateway>,

    /// "Gateway required"
    is_onchain_analysis_required: bool,

    profile_next_index_analyzer: Option<ProfileNextIndexAnalyzer>,
}

impl DeriveAndAnalyzeInput {
    /// # Panics
    /// Panics if some IDs of `ids_of_new_factor_sources` are not found in `factor_sources`
    pub fn new(
        factor_sources: IndexSet<HDFactorSource>,
        ids_of_new_factor_sources: IndexSet<FactorSourceIDFromHash>,
        cache: Option<PreDerivedKeysCache>,
        gateway: Arc<dyn Gateway>,
        is_onchain_analysis_required: bool,
        profile_next_index_analyzer: Option<ProfileNextIndexAnalyzer>,
    ) -> Self {
        assert!(
            ids_of_new_factor_sources
                .iter()
                .all(|id| factor_sources.iter().any(|f| f.factor_source_id() == *id)),
            "Discrepancy! Some IDs of new factor sources are not found in factor sources!"
        );

        Self {
            factor_sources,
            ids_of_new_factor_sources,
            cache,
            gateway,
            is_onchain_analysis_required,
            profile_next_index_analyzer,
        }
    }
}
