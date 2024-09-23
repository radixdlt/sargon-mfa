use crate::prelude::*;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct AnyFactorDerivationRequests(IndexSet<AnyFactorDerivationRequest>);

impl FromIterator<AnyFactorDerivationRequest> for AnyFactorDerivationRequests {
    fn from_iter<I: IntoIterator<Item = AnyFactorDerivationRequest>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl AnyFactorDerivationRequests {
    pub fn new(requests: IndexSet<AnyFactorDerivationRequest>) -> Self {
        Self(requests.into_iter().collect())
    }
    pub fn just(request: AnyFactorDerivationRequest) -> Self {
        Self(IndexSet::just(request))
    }

    pub fn merge(&mut self, other: Self) {
        self.0.extend(other.0);
    }

    /// TODO: Correct to do cartesian product: `N * M` many requests?
    pub fn for_each_factor_source(
        &self,
        factor_sources: FactorSources,
    ) -> IndexSet<DerivationRequest> {
        self.for_each_factor_source_id(
            factor_sources
                .factor_sources()
                .into_iter()
                .map(|f| f.factor_source_id())
                .collect(),
        )
    }

    pub fn for_each_factor_source_id(
        &self,
        factor_source_ids: IndexSet<FactorSourceIDFromHash>,
    ) -> IndexSet<DerivationRequest> {
        factor_source_ids
            .iter()
            .flat_map(|f| {
                self.0
                    .clone()
                    .into_iter()
                    .map(|x| x.derivation_request_with_factor_source_id(*f))
            })
            .collect()
    }
}
