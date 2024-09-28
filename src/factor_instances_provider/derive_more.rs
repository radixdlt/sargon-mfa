use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeriveMore {
    WithKnownStartIndex {
        with_start_index: DerivationRequestWithRange,
        number_of_instances_needed_to_fully_satisfy_request: Option<usize>,
    },
    WithoutKnownLastIndex {
        request: QuantifiedUnindexDerivationRequest,
        number_of_instances_needed_to_fully_satisfy_request: usize,
    },
}
impl DeriveMore {
    pub fn number_of_instances_to_use_directly(
        &self,
        original_purpose: FactorInstancesRequestPurpose,
    ) -> usize {
        match original_purpose {
            FactorInstancesRequestPurpose::MARS { .. } => {
                DerivationRequestQuantitySelector::FILL_CACHE_QUANTITY
            }
            FactorInstancesRequestPurpose::OARS { .. } => {
                DerivationRequestQuantitySelector::FILL_CACHE_QUANTITY
            }
            FactorInstancesRequestPurpose::UpdateOrSetSecurityShieldForAccounts {
                accounts,
                ..
            } => accounts.len(),
            FactorInstancesRequestPurpose::PreDeriveInstancesForNewFactorSource { .. } => 0,
            FactorInstancesRequestPurpose::NewVirtualUnsecurifiedAccount { .. } => 1,
        }
    }

    fn _number_of_instances_needed_to_fully_satisfy_request(&self) -> Option<usize> {
        match self {
            Self::WithKnownStartIndex {
                number_of_instances_needed_to_fully_satisfy_request,
                ..
            } => *number_of_instances_needed_to_fully_satisfy_request,
            Self::WithoutKnownLastIndex {
                number_of_instances_needed_to_fully_satisfy_request,
                ..
            } => Some(*number_of_instances_needed_to_fully_satisfy_request),
        }
    }

    pub fn unquantified(&self) -> UnquantifiedUnindexDerivationRequest {
        match self {
            Self::WithKnownStartIndex {
                with_start_index, ..
            } => with_start_index.clone().into(),
            Self::WithoutKnownLastIndex { request, .. } => request.clone().into(),
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

    pub fn maybe_some_to_use_directly(
        key: UnquantifiedUnindexDerivationRequest,
        to_cache: FactorInstances,
        to_use_directly: FactorInstances,
    ) -> Self {
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
