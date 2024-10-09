use std::ops::AddAssign;

use crate::{factor_instances_provider::agnostic_paths, prelude::*};

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

    fn max_account_veci(
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

    pub fn max_identity_veci(
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

    fn max_account_mfa(&self, factor_source_id: FactorSourceIDFromHash) -> Option<HDPathComponent> {
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

    fn max_identity_mfa(
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

    pub fn next(
        &self,
        agnostic_path: NetworkIndexAgnosticPath,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Option<HDPathComponent> {
        let last = if agnostic_path == NetworkIndexAgnosticPath::account_veci() {
            self.max_account_veci(factor_source_id)
        } else if agnostic_path == NetworkIndexAgnosticPath::account_mfa() {
            self.max_account_mfa(factor_source_id)
        } else if agnostic_path == NetworkIndexAgnosticPath::identity_mfa() {
            self.max_identity_mfa(factor_source_id)
        } else if agnostic_path == NetworkIndexAgnosticPath::identity_veci() {
            self.max_identity_veci(factor_source_id)
        } else {
            panic!("Unrecognized agnostic_path: {:?}", agnostic_path);
        };

        last.map(|l| l.add_one())
    }
}

#[derive(Debug)]
pub struct NextDerivationEntityIndexWithLocalOffsetsForFactorSource {
    #[allow(dead_code)]
    factor_source_id: FactorSourceIDFromHash,
    local_offsets: RwLock<HashMap<NetworkIndexAgnosticPath, HDPathValue>>,
}
impl NextDerivationEntityIndexWithLocalOffsetsForFactorSource {
    pub fn empty(factor_source_id: FactorSourceIDFromHash) -> Self {
        Self {
            factor_source_id,
            local_offsets: RwLock::new(HashMap::new()),
        }
    }
    pub fn reserve(&self, agnostic_path: NetworkIndexAgnosticPath) -> HDPathValue {
        let mut binding = self.local_offsets.write().unwrap();
        if let Some(existing) = binding.get_mut(&agnostic_path) {
            let free = *existing;
            existing.add_assign(1);
            free
        } else {
            binding.insert(agnostic_path, 1);
            0
        }
    }
}

#[derive(Default, Debug)]
pub struct NextDerivationEntityIndexWithLocalOffsets {
    local_offsets_per_factor_source: RwLock<
        HashMap<FactorSourceIDFromHash, NextDerivationEntityIndexWithLocalOffsetsForFactorSource>,
    >,
}

impl NextDerivationEntityIndexWithLocalOffsets {
    pub fn reserve(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        agnostic_path: NetworkIndexAgnosticPath,
    ) -> HDPathValue {
        let mut binding = self.local_offsets_per_factor_source.write().unwrap();
        if let Some(for_factor) = binding.get_mut(&factor_source_id) {
            for_factor.reserve(agnostic_path)
        } else {
            let new =
                NextDerivationEntityIndexWithLocalOffsetsForFactorSource::empty(factor_source_id);
            let next = new.reserve(agnostic_path);
            binding.insert(factor_source_id, new);
            next
        }
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
            local_offsets: NextDerivationEntityIndexWithLocalOffsets::default(),
        }
    }

    pub fn next(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        agnostic_path: NetworkIndexAgnosticPath,
    ) -> HDPathComponent {
        let default_for_profile = HDPathComponent::new_with_key_space_and_base_index(
            agnostic_path.key_space,
            U30::new(0).unwrap(),
        );
        let local = self.local_offsets.reserve(factor_source_id, agnostic_path);
        let from_profile = self
            .profile_analyzing
            .next(agnostic_path, factor_source_id)
            .unwrap_or(default_for_profile);

        let new = from_profile.add_n(local);

        println!(
            "🔮 from_profile: {}, from_local: {}, new: {}",
            from_profile, local, new
        );
        new
    }
}
