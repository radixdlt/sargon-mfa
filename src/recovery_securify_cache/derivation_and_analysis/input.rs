#![allow(unused)]

use crate::prelude::*;

#[async_trait::async_trait]
pub trait IsDerivationDoneQuery: Sync + Send {
    async fn is_derivation_done(&self, analysis: &IntermediaryDerivationAnalysis) -> Result<bool>;
}

pub struct DeriveAndAnalyzeInput {
    factor_sources: IndexSet<HDFactorSource>,
    ids_of_new_factor_sources: IndexSet<FactorSourceIDFromHash>,

    next_requests: DerivationRequests,

    factor_instances_provider: Arc<dyn IsFactorInstancesProvider>,

    /// Check if there is any known entity associated with a given factor instance,
    /// if so, some base info, if not, it is counted as "probably free".
    pub analyze_factor_instances: Arc<dyn IsIntermediaryDerivationAnalyzer>,
    pub is_done: Arc<dyn IsDerivationDoneQuery>,
}

impl DeriveAndAnalyzeInput {
    /// # Panics
    /// Panics if some IDs of `ids_of_new_factor_sources` are not found in `factor_sources`
    pub fn new(
        factor_sources: IndexSet<HDFactorSource>,
        ids_of_new_factor_sources: IndexSet<FactorSourceIDFromHash>,
        initial_derivation_requests: DerivationRequests,
        factor_instances_provider: Arc<dyn IsFactorInstancesProvider>,
        analyze_factor_instances: Arc<dyn IsIntermediaryDerivationAnalyzer>,
        is_done: Arc<dyn IsDerivationDoneQuery>,
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
            next_requests: initial_derivation_requests,
            factor_instances_provider,
            analyze_factor_instances,
            is_done,
        }
    }
}

#[async_trait::async_trait]
impl IsIntermediaryDerivationAnalyzer for DeriveAndAnalyzeInput {
    async fn analyze(
        &self,
        factor_instances: FactorInstances,
    ) -> Result<IntermediaryDerivationAnalysis> {
        self.analyze_factor_instances
            .analyze(factor_instances)
            .await
    }
}

#[async_trait::async_trait]
impl IsDerivationDoneQuery for DeriveAndAnalyzeInput {
    async fn is_derivation_done(&self, analysis: &IntermediaryDerivationAnalysis) -> Result<bool> {
        self.is_done.is_derivation_done(analysis).await
    }
}

impl DeriveAndAnalyzeInput {
    fn next_requests(&self) -> DerivationRequests {
        self.next_requests.clone()
    }

    pub async fn load_cached_or_derive_new_instances(&self) -> Result<FactorInstances> {
        let factor_sources = self.all_factor_sources();
        let requests = self.next_requests();
        let factor_instances = self
            .factor_instances_provider
            .provide_instances(requests)
            .await?;

        Ok(factor_instances)
    }
}

impl DeriveAndAnalyzeInput {
    pub fn all_factor_sources(&self) -> IndexSet<HDFactorSource> {
        self.factor_sources.clone().into_iter().collect()
    }

    pub fn new_factor_sources(&self) -> IndexSet<HDFactorSource> {
        self.all_factor_sources()
            .into_iter()
            .filter(|f| {
                !self
                    .ids_of_new_factor_sources
                    .contains(&f.factor_source_id())
            })
            .collect()
    }

    pub fn old_factor_sources(&self) -> IndexSet<HDFactorSource> {
        self.all_factor_sources()
            .into_iter()
            .filter(|f| {
                self.ids_of_new_factor_sources
                    .contains(&f.factor_source_id())
            })
            .collect()
    }
}
