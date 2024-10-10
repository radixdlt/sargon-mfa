use std::hash::Hash;

use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssertMatches {
    pub network_id: NetworkID,
    pub key_kind: CAP26KeyKind,
    pub entity_kind: CAP26EntityKind,
    pub key_space: KeySpace,
}
impl AssertMatches {
    pub fn matches(&self, path: &DerivationPath) -> DerivationPath {
        assert_eq!(self.entity_kind, path.entity_kind);
        assert_eq!(self.network_id, path.network_id);
        assert_eq!(self.entity_kind, path.entity_kind);
        assert_eq!(self.key_space, path.key_space());
        path.clone()
    }
}
impl MatrixOfFactorInstances {
    fn highest_derivation_path_index(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        assert_matches: AssertMatches,
    ) -> Option<HDPathComponent> {
        self.all_factors()
            .into_iter()
            .filter(|f| f.factor_source_id() == factor_source_id)
            .map(|f| f.derivation_path())
            .map(|p| assert_matches.matches(&p))
            .map(|p| p.index)
            .max()
    }
}
impl SecurifiedEntityControl {
    fn highest_derivation_path_index(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        assert_matches: AssertMatches,
    ) -> Option<HDPathComponent> {
        self.matrix
            .highest_derivation_path_index(factor_source_id, assert_matches)
    }
}

pub trait IsSecurifiedEntity:
    Hash + Eq + Clone + IsNetworkAware + TryFrom<AccountOrPersona> + Into<Self::BaseEntity>
{
    type BaseEntity: IsEntity + std::hash::Hash + Eq;
    fn kind() -> CAP26EntityKind {
        Self::BaseEntity::kind()
    }
    fn securified_entity_control(&self) -> SecurifiedEntityControl;

    fn new(
        name: impl AsRef<str>,
        address: <Self::BaseEntity as IsEntity>::Address,
        securified_entity_control: SecurifiedEntityControl,
        third_party_deposit: impl Into<Option<ThirdPartyDepositPreference>>,
    ) -> Self;

    fn highest_derivation_path_index(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        assert_matches: AssertMatches,
    ) -> Option<HDPathComponent> {
        self.securified_entity_control()
            .highest_derivation_path_index(factor_source_id, assert_matches)
    }
}
