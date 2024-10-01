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
            } => match self {
                Self::WithKnownStartIndex {
                    number_of_instances_needed_to_fully_satisfy_request,
                    ..
                } => number_of_instances_needed_to_fully_satisfy_request.unwrap_or(accounts.len()),
                Self::WithoutKnownLastIndex {
                    number_of_instances_needed_to_fully_satisfy_request,
                    ..
                } => *number_of_instances_needed_to_fully_satisfy_request,
            },
            FactorInstancesRequestPurpose::PreDeriveInstancesForNewFactorSource { .. } => 0,
            FactorInstancesRequestPurpose::NewVirtualUnsecurifiedAccount { .. } => 1,
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
