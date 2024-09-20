#![allow(unused)]
#![allow(unused_variables)]

use crate::prelude::*;

pub struct DeriveAndAnalyzeAccountRecoveryScanInput {
    factor_sources: IndexSet<HDFactorSource>,
    gateway: Arc<dyn Gateway>,
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
}

impl DeriveAndAnalyzeAccountRecoveryScanInput {
    pub fn new(
        factor_sources: IndexSet<HDFactorSource>,
        gateway: Arc<dyn Gateway>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        Self {
            factor_sources,
            gateway,
            derivation_interactors,
        }
    }
}

impl From<DeriveAndAnalyzeAccountRecoveryScanInput> for DeriveAndAnalyzeInput {
    #[allow(clippy::diverging_sub_expression)]
    fn from(value: DeriveAndAnalyzeAccountRecoveryScanInput) -> Self {
        let unfactored_derivation_requests = AnyFactorDerivationRequest::many_for_each_on(
            NetworkID::Mainnet,
            [CAP26EntityKind::Account],
            [CAP26KeyKind::TransactionSigning],
            [KeySpace::Securified, KeySpace::Unsecurified],
        );

        let initial_derivation_requests = value
            .factor_sources
            .clone()
            .into_iter()
            .flat_map(|f| {
                let factor_source_id = f.factor_source_id();
                unfactored_derivation_requests
                    .clone()
                    .into_iter()
                    .map(move |u| u.derivation_request_with_factor_source_id(factor_source_id))
            })
            .collect::<IndexSet<_>>();

        let factor_instances_provider: Arc<dyn IsFactorInstancesProvider> = { unreachable!() };
        let analyze_factor_instances: Arc<dyn IsIntermediaryDerivationAnalyzer> =
            { unreachable!() };

        Self::new(
            value.factor_sources.clone(),
            value
                .factor_sources
                .into_iter()
                .map(|f| f.factor_source_id())
                .collect(),
            initial_derivation_requests,
            factor_instances_provider,
            analyze_factor_instances,
        )
    }
}

pub struct UncachedFactorInstanceProvider {
    factor_sources: IndexSet<HDFactorSource>,
    derivation_index_ranges_start_values:
        IndexMap<FactorSourceIDFromHash, IndexMap<DerivationRequest, HDPathValue>>,
    interactors: Arc<dyn KeysDerivationInteractors>,
}

impl UncachedFactorInstanceProvider {
    fn derivation_paths_for_requests(
        &self,
        derivation_requests: IndexSet<DerivationRequest>,
    ) -> IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>> {
        todo!()
    }
    async fn derive_instances(
        &self,
        derivation_requests: IndexSet<DerivationRequest>,
    ) -> Result<IndexSet<HierarchicalDeterministicFactorInstance>> {
        let derivation_paths = self.derivation_paths_for_requests(derivation_requests);
        let keys_collector = KeysCollector::new(
            self.factor_sources.clone(),
            derivation_paths,
            self.interactors.clone(),
        )?;
        let derived = keys_collector.collect_keys().await;
        Ok(derived.all_factors())
    }
}

#[async_trait::async_trait]
impl IsFactorInstancesProvider for UncachedFactorInstanceProvider {
    async fn provide_instances(
        &self,
        derivation_requests: IndexSet<DerivationRequest>,
    ) -> Result<IndexSet<HierarchicalDeterministicFactorInstance>> {
        self.derive_instances(derivation_requests).await
    }
}
