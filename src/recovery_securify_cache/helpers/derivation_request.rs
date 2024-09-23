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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DerivationRequest {
    pub factor_source_id: FactorSourceIDFromHash,
    pub network_id: NetworkID,
    pub entity_kind: CAP26EntityKind,
    pub key_space: KeySpace,
    pub key_kind: CAP26KeyKind,
}

impl DerivationRequest {
    pub fn new(
        factor_source_id: FactorSourceIDFromHash,
        network_id: NetworkID,
        entity_kind: CAP26EntityKind,
        key_space: KeySpace,
        key_kind: CAP26KeyKind,
    ) -> Self {
        Self {
            factor_source_id,
            network_id,
            entity_kind,
            key_space,
            key_kind,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AnyFactorDerivationRequest {
    pub network_id: NetworkID,
    pub entity_kind: CAP26EntityKind,
    pub key_space: KeySpace,
    pub key_kind: CAP26KeyKind,
}

impl AnyFactorDerivationRequest {
    pub fn new(
        network_id: NetworkID,
        entity_kind: CAP26EntityKind,
        key_space: KeySpace,
        key_kind: CAP26KeyKind,
    ) -> Self {
        Self {
            network_id,
            entity_kind,
            key_kind,
            key_space,
        }
    }

    pub fn derivation_request_with_factor_source_id(
        &self,
        factor_source_id: FactorSourceIDFromHash,
    ) -> DerivationRequest {
        DerivationRequest::new(
            factor_source_id,
            self.network_id,
            self.entity_kind,
            self.key_space,
            self.key_kind,
        )
    }

    pub fn many_for_each_on(
        network_id: NetworkID,
        entity_kinds: impl IntoIterator<Item = CAP26EntityKind>,
        key_kinds: impl IntoIterator<Item = CAP26KeyKind>,
        key_spaces: impl IntoIterator<Item = KeySpace>,
    ) -> IndexSet<Self> {
        let entity_kinds = entity_kinds.into_iter().collect::<IndexSet<_>>();
        let key_kinds = key_kinds.into_iter().collect::<IndexSet<_>>();
        let key_spaces = key_spaces.into_iter().collect::<IndexSet<_>>();

        let mut requests = IndexSet::<Self>::new();

        for entity_kind in entity_kinds.into_iter() {
            for key_kind in key_kinds.clone().into_iter() {
                for key_space in key_spaces.clone().into_iter() {
                    let request = Self::new(network_id, entity_kind, key_space, key_kind);
                    requests.insert(request);
                }
            }
        }
        requests
    }
}
