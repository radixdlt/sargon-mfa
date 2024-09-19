use crate::prelude::*;

/// A request that cannot be fulfilled, and the reason why.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DerivationRequestUnfulfillableByCache {
    /// The request which cannot be fulfilled.
    pub request: DerivationRequest,

    /// The reason why `request` could not be fulfilled.
    pub reason: DerivationRequestUnfulfillableByCacheReason,
}

impl DerivationRequestUnfulfillableByCache {
    pub fn factor_source_id(&self) -> FactorSourceIDFromHash {
        self.request.factor_source_id
    }
    pub fn empty(request: DerivationRequest) -> Self {
        Self {
            request,
            reason: DerivationRequestUnfulfillableByCacheReason::Empty,
        }
    }

    /// # Panics
    /// Panics if `last_factor` does not share same parameters as `request`
    pub fn last(
        request: DerivationRequest,
        last_factor: &HierarchicalDeterministicFactorInstance,
    ) -> Self {
        assert!(
            last_factor.matches(&request),
            "last_factor must match request"
        );
        Self {
            request,
            reason: DerivationRequestUnfulfillableByCacheReason::Last(
                last_factor.derivation_path().index,
            ),
        }
    }

    pub fn is_reason_empty(&self) -> bool {
        matches!(
            self.reason,
            DerivationRequestUnfulfillableByCacheReason::Empty
        )
    }

    pub fn is_reason_last(&self) -> bool {
        matches!(
            self.reason,
            DerivationRequestUnfulfillableByCacheReason::Last(_)
        )
    }
}

impl HierarchicalDeterministicFactorInstance {
    pub fn matches(&self, request: &DerivationRequest) -> bool {
        self.factor_source_id() == request.factor_source_id
            && self.derivation_path().matches(request)
    }
}
impl DerivationPath {
    fn matches(&self, request: &DerivationRequest) -> bool {
        self.network_id == request.network_id
            && self.entity_kind == request.entity_kind
            && self.key_kind == request.key_kind
            && self.index.key_space() == request.key_space
    }
}
