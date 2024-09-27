use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeriveMoreToSatisfyOriginalRequest {
    WithKnownStartIndex {
        with_start_index: DerivationRequestWithRange,
        number_of_instances_needed_to_fully_satisfy_request: Option<usize>,
    },
    /// When cache is empty, we don't know the last index, in this context,
    /// we will need to use the NextIndexAssigner.
    WithoutKnownLastIndex {
        without_known_last_index: QuantifiedUnindexDerivationRequest,
        requested_quantity: usize,
    },
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
            Self::WithoutKnownLastIndex {
                requested_quantity, ..
            } => Some(*requested_quantity),
        }
    }
    pub fn unquantified(&self) -> UnquantifiedUnindexDerivationRequest {
        match self {
            Self::WithKnownStartIndex {
                with_start_index, ..
            } => with_start_index.clone().into(),
            Self::WithoutKnownLastIndex {
                without_known_last_index,
                ..
            } => without_known_last_index.clone().into(),
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
