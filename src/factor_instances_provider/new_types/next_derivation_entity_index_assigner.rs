use std::ops::AddAssign;

use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AssertMatches {
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
impl SecurifiedEntity {
    fn highest_derivation_path_index(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        assert_matches: AssertMatches,
    ) -> Option<HDPathComponent> {
        self.securified_entity_control()
            .highest_derivation_path_index(factor_source_id, assert_matches)
    }
}

pub struct NextDerivationEntityIndexProfileAnalyzingAssigner {
    network_id: NetworkID,

    /// might be empty
    unsecurified_accounts_on_network: IndexSet<UnsecurifiedEntity>,

    /// might be empty
    securified_accounts_on_network: IndexSet<SecurifiedEntity>,

    /// might be empty
    unsecurified_identities_on_network: IndexSet<UnsecurifiedEntity>,

    /// might be empty
    securified_identities_on_network: IndexSet<SecurifiedEntity>,
}

impl NextDerivationEntityIndexProfileAnalyzingAssigner {
    pub fn new(network_id: NetworkID, profile: Option<Profile>) -> Self {
        let unsecurified_accounts_on_network = profile
            .as_ref()
            .map(|p| p.unsecurified_accounts_on_network(network_id))
            .unwrap_or_default();

        let securified_accounts_on_network = profile
            .as_ref()
            .map(|p| p.securified_accounts_on_network(network_id))
            .unwrap_or_default();

        let unsecurified_identities_on_network = profile
            .as_ref()
            .map(|p| p.unsecurified_identities_on_network(network_id))
            .unwrap_or_default();

        let securified_identities_on_network = profile
            .as_ref()
            .map(|p| p.securified_identities_on_network(network_id))
            .unwrap_or_default();

        Self {
            network_id,
            unsecurified_accounts_on_network,
            securified_accounts_on_network,
            unsecurified_identities_on_network,
            securified_identities_on_network,
        }
    }

    pub fn next_account_veci(
        &self,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Option<HDPathComponent> {
        self.unsecurified_accounts_on_network
            .clone()
            .into_iter()
            .map(|x: UnsecurifiedEntity| x.veci().factor_instance())
            .filter(|f| f.factor_source_id == factor_source_id)
            .map(|f| f.derivation_path())
            .map(|p| {
                AssertMatches {
                    network_id: self.network_id,
                    key_kind: CAP26KeyKind::TransactionSigning,
                    entity_kind: CAP26EntityKind::Account,
                    key_space: KeySpace::Unsecurified,
                }
                .matches(&p)
            })
            .map(|fi| fi.index)
            .max()
    }

    pub fn next_identity_veci(
        &self,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Option<HDPathComponent> {
        self.unsecurified_identities_on_network
            .clone()
            .into_iter()
            .map(|x: UnsecurifiedEntity| x.veci().factor_instance())
            .filter(|f| f.factor_source_id == factor_source_id)
            .map(|f| f.derivation_path())
            .map(|p| {
                AssertMatches {
                    network_id: self.network_id,
                    key_kind: CAP26KeyKind::TransactionSigning,
                    entity_kind: CAP26EntityKind::Identity,
                    key_space: KeySpace::Unsecurified,
                }
                .matches(&p)
            })
            .map(|fi| fi.index)
            .max()
    }

    pub fn next(
        &self,
        template: DerivationTemplate,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Option<HDPathComponent> {
        match template {
            DerivationTemplate::AccountVeci => self.next_account_veci(factor_source_id),
            DerivationTemplate::AccountMfa => self.next_account_mfa(factor_source_id),
            DerivationTemplate::IdentityVeci => self.next_identity_veci(factor_source_id),
            DerivationTemplate::IdentityMfa => self.next_identity_mfa(factor_source_id),
        }
    }

    pub fn next_account_mfa(
        &self,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Option<HDPathComponent> {
        self.securified_accounts_on_network
            .clone()
            .into_iter()
            .flat_map(|e: SecurifiedEntity| {
                e.highest_derivation_path_index(
                    factor_source_id,
                    AssertMatches {
                        network_id: self.network_id,
                        key_kind: CAP26KeyKind::TransactionSigning,
                        entity_kind: CAP26EntityKind::Account,
                        key_space: KeySpace::Securified,
                    },
                )
            })
            .max()
    }

    pub fn next_identity_mfa(
        &self,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Option<HDPathComponent> {
        self.securified_identities_on_network
            .clone()
            .into_iter()
            .flat_map(|e: SecurifiedEntity| {
                e.highest_derivation_path_index(
                    factor_source_id,
                    AssertMatches {
                        network_id: self.network_id,
                        key_kind: CAP26KeyKind::TransactionSigning,
                        entity_kind: CAP26EntityKind::Identity,
                        key_space: KeySpace::Securified,
                    },
                )
            })
            .max()
    }
}

#[derive(Debug)]
pub struct NextDerivationEntityIndexWithLocalOffsetsForFactorSource {
    #[allow(dead_code)]
    network_id: NetworkID,
    #[allow(dead_code)]
    factor_source_id: FactorSourceIDFromHash,
    local_offsets_derivation_template_query: RwLock<HashMap<DerivationTemplate, HDPathValue>>,
}
impl NextDerivationEntityIndexWithLocalOffsetsForFactorSource {
    pub fn empty(network_id: NetworkID, factor_source_id: FactorSourceIDFromHash) -> Self {
        Self {
            network_id,
            factor_source_id,
            local_offsets_derivation_template_query: RwLock::new(HashMap::new()),
        }
    }
    pub fn reserve(&self, template: DerivationTemplate) -> HDPathValue {
        let mut binding = self
            .local_offsets_derivation_template_query
            .write()
            .unwrap();
        if let Some(existing) = binding.get_mut(&template) {
            let free = *existing;
            existing.add_assign(1);
            free
        } else {
            let next_free = 1;
            binding.insert(template, next_free);
            0
        }
    }
}

#[derive(Debug)]
pub struct NextDerivationEntityIndexWithLocalOffsets {
    network_id: NetworkID,
    local_offsets_per_factor_source: RwLock<
        HashMap<FactorSourceIDFromHash, NextDerivationEntityIndexWithLocalOffsetsForFactorSource>,
    >,
}

impl NextDerivationEntityIndexWithLocalOffsets {
    pub fn empty(network_id: NetworkID) -> Self {
        Self {
            network_id,
            local_offsets_per_factor_source: RwLock::new(HashMap::new()),
        }
    }
    pub fn reserve(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        template: DerivationTemplate,
    ) -> HDPathValue {
        let default = NextDerivationEntityIndexWithLocalOffsetsForFactorSource::empty(
            self.network_id,
            factor_source_id,
        );
        let binding = self.local_offsets_per_factor_source.write().unwrap();
        let for_factor = binding.get(&factor_source_id).unwrap_or(&default);

        for_factor.reserve(template)
    }
}

pub struct NextDerivationEntityIndexAssigner {
    #[allow(dead_code)]
    network_id: NetworkID,
    profile_analyzing: NextDerivationEntityIndexProfileAnalyzingAssigner,
    local_offsets: NextDerivationEntityIndexWithLocalOffsets,
}

impl NextDerivationEntityIndexAssigner {
    pub fn new(network_id: NetworkID, profile: Option<Profile>) -> Self {
        let profile_analyzing =
            NextDerivationEntityIndexProfileAnalyzingAssigner::new(network_id, profile);
        Self {
            network_id,
            profile_analyzing,
            local_offsets: NextDerivationEntityIndexWithLocalOffsets::empty(network_id),
        }
    }

    pub fn next(
        &self,
        template: DerivationTemplate,
        factor_source_id: FactorSourceIDFromHash,
    ) -> HDPathComponent {
        let default_for_profile = HDPathComponent::new_with_key_space_and_base_index(
            template.key_space(),
            U30::new(0).unwrap(),
        );
        let local = self.local_offsets.reserve(factor_source_id, template);
        let from_profile = self
            .profile_analyzing
            .next(template, factor_source_id)
            .unwrap_or(default_for_profile);

        from_profile.add_n(local)
    }
    pub fn next_account_veci(&self, factor_source_id: FactorSourceIDFromHash) -> HDPathComponent {
        self.next(DerivationTemplate::AccountVeci, factor_source_id)
    }
}
