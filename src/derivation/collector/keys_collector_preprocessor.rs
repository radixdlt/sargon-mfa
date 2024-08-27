use crate::prelude::*;
pub struct KeysCollectorPreprocessor {
    derivation_paths: IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>,
}

impl KeysCollectorPreprocessor {
    pub fn new(
        derivation_paths: IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>,
    ) -> Self {
        Self { derivation_paths }
    }

    pub(crate) fn preprocess(
        &self,
        all_factor_sources_in_profile: IndexSet<HDFactorSource>,
    ) -> (KeysCollectorState, IndexSet<FactorSourcesOfKind>) {
        let all_factor_sources_in_profile = all_factor_sources_in_profile
            .into_iter()
            .map(|f| (f.factor_source_id(), f))
            .collect::<HashMap<FactorSourceIDFromHash, HDFactorSource>>();

        let factor_sources_of_kind = sort_group_factors(
            self.derivation_paths
                .clone()
                .keys()
                .map(|id| {
                    all_factor_sources_in_profile
                        .get(id)
                        .expect("Should have all factor sources")
                        .clone()
                })
                .collect::<HashSet<_>>(),
        );
        let state = KeysCollectorState::new(self.derivation_paths.clone());
        (state, factor_sources_of_kind)
    }
}
