use std::ops::AddAssign;

use crate::{factor_instances_provider::agnostic_paths, prelude::*};

pub struct NextDerivationEntityIndexProfileAnalyzingAssigner {
    network_id: NetworkID,

    /// might be empty
    unsecurified_accounts_on_network: IndexSet<UnsecurifiedEntity>,

    /// might be empty
    securified_accounts_on_network: IndexSet<SecurifiedAccount>,

    /// might be empty
    unsecurified_identities_on_network: IndexSet<UnsecurifiedEntity>,

    /// might be empty
    securified_identities_on_network: IndexSet<SecurifiedPersona>,
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
            .flat_map(|e: SecurifiedAccount| {
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
            .flat_map(|e: SecurifiedPersona| {
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

pub enum OffsetFromCache {
    /// Finding max amongst already loaded (and removed) from cache, saved
    /// locally
    FindMaxInRemoved {
        pf_found_in_cache: IndexMap<FactorSourceIDFromHash, FactorInstances>,
    },
    /// Known max by having peeked into the cache earlier.
    KnownMax {
        instance: HierarchicalDeterministicFactorInstance,
    },
}
impl OffsetFromCache {
    pub fn next(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        index_agnostic_path: IndexAgnosticPath,
    ) -> Option<HDPathComponent> {
        self._max(factor_source_id, index_agnostic_path)
            .map(|i| i.add_one())
    }
    fn _max(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        index_agnostic_path: IndexAgnosticPath,
    ) -> Option<HDPathComponent> {
        match self {
            Self::FindMaxInRemoved { pf_found_in_cache } => pf_found_in_cache
                .get(&factor_source_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|f| f.agnostic_path() == index_agnostic_path)
                .map(|f| f.derivation_path().index)
                .max(),
            Self::KnownMax { instance } => {
                assert_eq!(instance.factor_source_id(), factor_source_id);
                assert_eq!(instance.agnostic_path(), index_agnostic_path);
                Some(instance.derivation_path().index)
            }
        }
    }
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
        index_agnostic_path: IndexAgnosticPath,
        cache_offset: OffsetFromCache,
    ) -> HDPathComponent {
        let default_index = HDPathComponent::new_with_key_space_and_base_index(
            index_agnostic_path.key_space,
            U30::new(0).unwrap(),
        );

        // Must update local offset based on values found in cache.
        // Imagine we are securifying 3 accounts with a single FactorSource
        // `L` to keep things simple, profile already contains 28 securified
        // accounts controlled by `L`, with the highest entity index is `27^`
        // We look for keys in the cache for `L` and we find 2, with entity
        // indices `[28^, 29^]`, so we need to derive 2 (+CACHE_FILLING_QUANTITY)
        // more keys. The next index assigner will correctly use a profile based offset
        // of 28^ for `L`, since it found the max value `28^` in Profile controlled by `L`.
        // If we would use `next` now, the index would be `next = max + 1`, and
        // `max = offset_from_profile + local_offset` = `28^ + 0^` = 28^.
        // Which is wrong! Since the cache contains `28^` and `29^`, we should
        // derive `2 (+CACHE_FILLING_QUANTITY)` starting at `30^`.
        let next_from_cache = cache_offset
            .next(factor_source_id, index_agnostic_path)
            .unwrap_or(default_index);
        let network_agnostic_path = index_agnostic_path.network_agnostic();
        let local = self
            .local_offsets
            .reserve(factor_source_id, network_agnostic_path);
        let next_from_profile = self
            .profile_analyzing
            .next(network_agnostic_path, factor_source_id)
            .unwrap_or(default_index);

        let max_index = std::cmp::max(next_from_profile, next_from_cache);

        max_index.add_n(local)
    }
}
