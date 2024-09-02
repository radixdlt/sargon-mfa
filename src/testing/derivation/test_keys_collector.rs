#![cfg(test)]
#![allow(unused)]

use crate::prelude::*;

pub(crate) struct TestDerivationInteractors {
    pub(crate) poly: Arc<dyn PolyFactorKeyDerivationInteractor + Send + Sync>,
    pub(crate) mono: Arc<dyn MonoFactorKeyDerivationInteractor + Send + Sync>,
}
impl TestDerivationInteractors {
    pub(crate) fn new(
        poly: impl PolyFactorKeyDerivationInteractor + Send + Sync + 'static,
        mono: impl MonoFactorKeyDerivationInteractor + Send + Sync + 'static,
    ) -> Self {
        Self {
            poly: Arc::new(poly),
            mono: Arc::new(mono),
        }
    }
}

impl TestDerivationInteractors {
    pub(crate) fn fail() -> Self {
        Self::new(
            TestDerivationParallelInteractor::fail(),
            TestDerivationSerialInteractor::fail(),
        )
    }
}
impl Default for TestDerivationInteractors {
    fn default() -> Self {
        Self::new(
            TestDerivationParallelInteractor::default(),
            TestDerivationSerialInteractor::default(),
        )
    }
}

impl KeysDerivationInteractors for TestDerivationInteractors {
    fn interactor_for(&self, kind: FactorSourceKind) -> KeyDerivationInteractor {
        match kind {
            FactorSourceKind::Device => KeyDerivationInteractor::poly(self.poly.clone()),
            _ => KeyDerivationInteractor::mono(self.mono.clone()),
        }
    }
}

pub(crate) struct TestDerivationParallelInteractor {
    handle: fn(
        MonoFactorKeyDerivationRequest,
    ) -> Result<IndexSet<HierarchicalDeterministicFactorInstance>>,
}
impl TestDerivationParallelInteractor {
    pub(crate) fn new(
        handle: fn(
            MonoFactorKeyDerivationRequest,
        ) -> Result<IndexSet<HierarchicalDeterministicFactorInstance>>,
    ) -> Self {
        Self { handle }
    }
    pub(crate) fn fail() -> Self {
        Self::new(|_| Err(CommonError::Failure))
    }
    fn derive(
        &self,
        request: MonoFactorKeyDerivationRequest,
    ) -> Result<IndexSet<HierarchicalDeterministicFactorInstance>> {
        (self.handle)(request)
    }
}
impl Default for TestDerivationParallelInteractor {
    fn default() -> Self {
        Self::new(do_derive_serially)
    }
}

fn do_derive_serially(
    request: MonoFactorKeyDerivationRequest,
) -> Result<IndexSet<HierarchicalDeterministicFactorInstance>> {
    let factor_source_id = &request.factor_source_id;
    let instances = request
        .derivation_paths
        .into_iter()
        .map(|p| HierarchicalDeterministicFactorInstance::mocked_with(p, factor_source_id))
        .collect::<IndexSet<_>>();

    Ok(instances)
}

#[async_trait::async_trait]
impl PolyFactorKeyDerivationInteractor for TestDerivationParallelInteractor {
    async fn derive(
        &self,
        request: PolyFactorKeyDerivationRequest,
    ) -> Result<KeyDerivationResponse> {
        let pairs_result: Result<
            IndexMap<FactorSourceIDFromHash, IndexSet<HierarchicalDeterministicFactorInstance>>,
        > = request
            .per_factor_source
            .into_iter()
            .map(|(k, r)| {
                let instances = self.derive(r);
                instances.map(|i| (k, i))
            })
            .collect();
        let pairs = pairs_result?;
        Ok(KeyDerivationResponse::new(pairs))
    }
}

pub(crate) struct TestDerivationSerialInteractor {
    handle: fn(
        MonoFactorKeyDerivationRequest,
    ) -> Result<IndexSet<HierarchicalDeterministicFactorInstance>>,
}
impl TestDerivationSerialInteractor {
    pub(crate) fn new(
        handle: fn(
            MonoFactorKeyDerivationRequest,
        ) -> Result<IndexSet<HierarchicalDeterministicFactorInstance>>,
    ) -> Self {
        Self { handle }
    }
    pub(crate) fn fail() -> Self {
        Self::new(|_| Err(CommonError::Failure))
    }
    fn derive(
        &self,
        request: MonoFactorKeyDerivationRequest,
    ) -> Result<IndexSet<HierarchicalDeterministicFactorInstance>> {
        (self.handle)(request)
    }
}
impl Default for TestDerivationSerialInteractor {
    fn default() -> Self {
        Self::new(do_derive_serially)
    }
}

#[async_trait::async_trait]
impl MonoFactorKeyDerivationInteractor for TestDerivationSerialInteractor {
    async fn derive(
        &self,
        request: MonoFactorKeyDerivationRequest,
    ) -> Result<KeyDerivationResponse> {
        let instances = self.derive(request.clone())?;
        Ok(KeyDerivationResponse::new(IndexMap::from_iter([(
            request.factor_source_id,
            instances,
        )])))
    }
}

impl KeysCollector {
    pub(crate) fn new_test_with_factor_sources(
        all_factor_sources_in_profile: impl IntoIterator<Item = HDFactorSource>,
        derivation_paths: impl IntoIterator<Item = (FactorSourceIDFromHash, IndexSet<DerivationPath>)>,
    ) -> Self {
        sensible_env_logger::safe_init!();
        Self::new(
            all_factor_sources_in_profile,
            derivation_paths.into_iter().collect(),
            Arc::new(TestDerivationInteractors::default()),
        )
        .unwrap()
    }

    pub(crate) fn new_test(
        derivation_paths: impl IntoIterator<Item = (FactorSourceIDFromHash, IndexSet<DerivationPath>)>,
    ) -> Self {
        Self::new_test_with_factor_sources(HDFactorSource::all(), derivation_paths)
    }

    pub(crate) fn with(
        factor_source: &HDFactorSource,
        network_id: NetworkID,
        key_kind: CAP26KeyKind,
        entity_kind: CAP26EntityKind,
        key_space: KeySpace,
    ) -> Self {
        let indices = StatelessDummyIndices;
        let path = indices.next_derivation_path(network_id, key_kind, entity_kind, key_space);
        Self::new_test_with_factor_sources(
            [factor_source.clone()],
            [(
                factor_source.factor_source_id(),
                IndexSet::from_iter([path]),
            )],
        )
    }
}
