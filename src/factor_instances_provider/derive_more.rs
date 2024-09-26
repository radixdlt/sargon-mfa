use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeriveMoreWithAbundance {
    requests_to_satisfy_original_request: IndexSet<DeriveMoreToSatisfyOriginalRequest>,
    /// should be standard batch size - length of `requests_to_satisfy_original_request`
    with_abundance: AnyFactorDerivationRequest,
}
impl DeriveMoreWithAbundance {
    pub fn new(
        requests_to_satisfy_original_request: IndexSet<DeriveMoreToSatisfyOriginalRequest>,
        purpose: &FactorInstancesRequestPurpose,
        next_index_assigner: &NextIndexAssignerWithEphemeralLocalOffsets,
    ) -> Self {
        let filling_cache_unindexed = purpose
            ._requests_with_quantity(DerivationRequestQuantitySelector::fill_cache_if_needed());

        let filling_cache_indexed = filling_cache_unindexed.

        Self {
            requests_to_satisfy_original_request,
            with_abundance: AnyFactorDerivationRequest::new(purpose),
        }
    }
    pub fn all_requests(self) -> IndexSet<DeriveMoreToSatisfyOriginalRequest> {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeriveMoreToSatisfyOriginalRequest {
    WithKnownStartIndex {
        with_start_index: DerivationRequestWithRange,
        number_of_instances_needed_to_fully_satisfy_request: Option<usize>,
    },
    /// When cache is empty, we don't know the last index, in this context,
    /// we will need to use the NextIndexAssigner.
    WithoutKnownLastIndex(QuantifiedUnindexDerivationRequest),
}
impl DeriveMoreToSatisfyOriginalRequest {
    /// `None` for `WithoutKnownLastIndex`, only `Some` for `WithKnownStartIndex`
    ///  where `if_partial_how_many_to_use_directly` is `Some`
    pub fn number_of_instances_needed_to_fully_satisfy_request(&self) -> Option<usize> {
        match self {
            Self::WithKnownStartIndex {
                number_of_instances_needed_to_fully_satisfy_request,
                ..
            } => *number_of_instances_needed_to_fully_satisfy_request,
            Self::WithoutKnownLastIndex(_) => None,
        }
    }
    pub fn unquantified(&self) -> UnquantifiedUnindexDerivationRequest {
        match self {
            Self::WithKnownStartIndex {
                with_start_index, ..
            } => with_start_index.clone().into(),
            Self::WithoutKnownLastIndex(request) => request.clone().into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NewlyDerived {
    key: UnquantifiedUnindexDerivationRequest,
    /// never empty
    to_cache: FactorInstances,
    /// can be empty
    pub to_use_directly: FactorInstances,
}
impl NewlyDerived {
    pub fn cache_all(key: UnquantifiedUnindexDerivationRequest, to_cache: FactorInstances) -> Self {
        Self::new(key, to_cache, FactorInstances::default())
    }

    /// # Panics if `to_cache` or to `to_use_directly` is empty.
    pub fn some_to_use_directly(
        key: UnquantifiedUnindexDerivationRequest,
        to_cache: FactorInstances,
        to_use_directly: FactorInstances,
    ) -> Self {
        assert!(!to_use_directly.is_empty());
        Self::new(key, to_cache, to_use_directly)
    }
    /// # Panics
    /// Panics if `to_cache` is empty.
    /// Also panics if any FactorInstances does not match the key.
    fn new(
        key: UnquantifiedUnindexDerivationRequest,
        to_cache: FactorInstances,
        to_use_directly: FactorInstances,
    ) -> Self {
        assert!(to_cache
            .factor_instances()
            .iter()
            .all(|factor_instance| { factor_instance.satisfies(key.clone()) }));

        assert!(to_use_directly
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
        (self.key.clone(), self.to_cache.clone())
    }
}
