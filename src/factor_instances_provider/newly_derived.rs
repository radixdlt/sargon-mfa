use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ToUseDirectly(pub FactorInstances);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ToCache(pub FactorInstances);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NewlyDerived {
    key: UnquantifiedUnindexDerivationRequest,
    /// never empty
    to_cache: ToCache,
    /// can be empty
    pub to_use_directly: ToUseDirectly,
}
impl NewlyDerived {
    pub fn cache_all(key: UnquantifiedUnindexDerivationRequest, to_cache: ToCache) -> Self {
        Self::new(key, to_cache, ToUseDirectly(FactorInstances::default()))
    }

    pub fn maybe_some_to_use_directly(
        key: UnquantifiedUnindexDerivationRequest,
        to_cache: ToCache,
        to_use_directly: ToUseDirectly,
    ) -> Self {
        Self::new(key, to_cache, to_use_directly)
    }

    /// # Panics
    /// Panics if `to_cache` is empty.
    /// Also panics if any FactorInstances does not match the key.
    fn new(
        key: UnquantifiedUnindexDerivationRequest,
        to_cache: ToCache,
        to_use_directly: ToUseDirectly,
    ) -> Self {
        assert!(to_cache
            .0
            .factor_instances()
            .iter()
            .all(|factor_instance| { factor_instance.satisfies(key.clone()) }));

        assert!(to_use_directly
            .0
            .factor_instances()
            .iter()
            .all(|factor_instance| { factor_instance.satisfies(key.clone()) }));

        Self {
            key,
            to_cache,
            to_use_directly,
        }
    }
    pub fn key_value_for_cache(&self) -> (UnquantifiedUnindexDerivationRequest, FactorInstances) {
        (self.key.clone(), self.to_cache.0.clone())
    }
}

impl HierarchicalDeterministicFactorInstance {
    pub fn satisfies(&self, request: UnquantifiedUnindexDerivationRequest) -> bool {
        self.derivation_path().satisfies(request.clone())
            && request.factor_source_id == self.factor_source_id
    }
}

impl DerivationPath {
    #[allow(clippy::nonminimal_bool)]
    fn satisfies(&self, request: impl Into<UnquantifiedUnindexDerivationRequest>) -> bool {
        let request = request.into();
        request.entity_kind == self.entity_kind
            && request.network_id == self.network_id
            && request.entity_kind == self.entity_kind
            && request.key_kind == self.key_kind
            && request.key_space == self.index.key_space()
    }
}
