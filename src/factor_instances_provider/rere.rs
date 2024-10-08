use std::clone;

use crate::prelude::*;

/// A DerivationPath that is not on any specified
/// network and which is not indexed.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct IndexAgnosticPath {
    pub network_id: NetworkID,
    pub entity_kind: CAP26EntityKind,
    pub key_kind: CAP26KeyKind,
    pub key_space: KeySpace,
}
impl From<(NetworkID, NetworkIndexAgnosticPath)> for IndexAgnosticPath {
    fn from((network_id, agnostic_path): (NetworkID, NetworkIndexAgnosticPath)) -> Self {
        Self {
            network_id,
            entity_kind: agnostic_path.entity_kind,
            key_kind: agnostic_path.key_kind,
            key_space: agnostic_path.key_space,
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct AbstractQuantifiedNetworkIndexAgnosticPath<T> {
    pub agnostic_path: NetworkIndexAgnosticPath,
    pub quantity: T,
}
pub type QuantifiedNetworkIndexAgnosticPath = AbstractQuantifiedNetworkIndexAgnosticPath<usize>;
pub type CacheFillingPlaceholderQuantityNetworkIndexAgnosticPath =
    AbstractQuantifiedNetworkIndexAgnosticPath<CacheFillingPlaceholderQuantity>;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum CacheFillingPlaceholderQuantity {
    OnlyCacheFilling,
    ToFulfillRequestAlsoCacheFilling {
        quantity_remaining_not_satisfied_by_cache: usize,
    },
}

/// Used as "presets"
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct NetworkIndexAgnosticPath {
    pub entity_kind: CAP26EntityKind,
    pub key_kind: CAP26KeyKind,
    pub key_space: KeySpace,
}
impl NetworkIndexAgnosticPath {
    fn new(entity_kind: CAP26EntityKind, key_kind: CAP26KeyKind, key_space: KeySpace) -> Self {
        Self {
            entity_kind,
            key_kind,
            key_space,
        }
    }
    fn transaction_signing(entity_kind: CAP26EntityKind, key_space: KeySpace) -> Self {
        Self::new(entity_kind, CAP26KeyKind::TransactionSigning, key_space)
    }
    pub fn account_veci() -> Self {
        Self::transaction_signing(CAP26EntityKind::Account, KeySpace::Unsecurified)
    }
    pub fn account_mfa() -> Self {
        Self::transaction_signing(CAP26EntityKind::Account, KeySpace::Securified)
    }
    pub fn identity_veci() -> Self {
        Self::transaction_signing(CAP26EntityKind::Identity, KeySpace::Unsecurified)
    }
    pub fn identity_mfa() -> Self {
        Self::transaction_signing(CAP26EntityKind::Identity, KeySpace::Securified)
    }
    pub fn all_presets() -> IndexSet<Self> {
        IndexSet::from_iter([
            Self::account_mfa(),
            Self::account_mfa(),
            Self::identity_veci(),
            Self::identity_mfa(),
        ])
    }
}

pub struct KeyValue<T> {
    /// PER FactorSource PER IndexAgnosticPath some value T
    pub values: HashMap<FactorSourceIDFromHash, HashMap<IndexAgnosticPath, T>>,
}

enum QuantityOutcome {
    Empty,
    Partial {
        instances: FactorInstances,
        remaining: usize,
    },
    Full {
        instances: FactorInstances,
    },
}
impl Cache {
    fn __remove(
        &mut self,
        factor_source_id: &FactorSourceIDFromHash,
        index_agnostic_path: &IndexAgnosticPath,
    ) -> FactorInstances {
        if let Some(cached_for_factor) = self.values.get_mut(factor_source_id) {
            if let Some(found_cached) = cached_for_factor.remove(index_agnostic_path) {
                return found_cached;
            }
        }
        FactorInstances::default()
    }

    fn remove(
        &mut self,
        factor_source_id: &FactorSourceIDFromHash,
        index_agnostic_path: &IndexAgnosticPath,
        quantity: usize,
    ) -> QuantityOutcome {
        let instances = self.__remove(factor_source_id, index_agnostic_path);
        if instances.is_empty() {
            return QuantityOutcome::Empty;
        }
        let len = instances.len();
        if len == quantity {
            return QuantityOutcome::Full { instances };
        }
        if len < quantity {
            return QuantityOutcome::Partial {
                instances,
                remaining: quantity - len,
            };
        }
        assert!(len > quantity);
        // need to split
        let instances = instances.factor_instances().into_iter().collect_vec();
        let (to_use, to_put_back) = instances.split_at(quantity);
        let to_put_back = FactorInstances::from_iter(to_put_back.iter().cloned());
        if let Some(cached_for_factor) = self.values.get_mut(factor_source_id) {
            cached_for_factor.insert(index_agnostic_path.clone(), to_put_back);
        }

        QuantityOutcome::Full {
            instances: FactorInstances::from_iter(to_use.iter().cloned()),
        }
    }
}

pub type Cache = KeyValue<FactorInstances>;

pub type LocalOffsets = KeyValue<u32>;

pub struct NextIndexAssigner {
    profile: Profile,
    local: LocalOffsets,
}

pub struct FactorInstancesProvider {
    cache: Cache,
    next_index_assigner: NextIndexAssigner,
}

pub struct KeysCollector;
impl KeysCollector {
    fn new(
        factor_sources: FactorSources,
        paths_per_factor: IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>,
    ) -> Self {
        todo!()
    }
    fn derive(self) -> Result<FactorInstances> {
        todo!()
    }
}

struct FactorInstancesProviderOutcome {
    /// Might be empty
    pub to_cache: FactorInstances,
    /// Might be empty
    pub to_use_directly: FactorInstances,

    /// LESS IMPORTANT - for tests...
    /// might overlap with `to_use_directly`
    pub _found_in_cache: FactorInstances,
    /// might overlap with `to_cache` and `to_use_directly`
    pub _newly_derived: FactorInstances,
}
impl FactorInstancesProvider {
    pub fn provide(self) -> Result<FactorInstancesProviderOutcome> {
        todo!()
    }
}

impl FactorInstancesProvider {
    pub fn for_account_mfa(
        cache: &mut Cache,
        matrix_of_factor_sources: MatrixOfFactorSources,
        profile: Profile,
        accounts: IndexSet<AccountAddress>,
    ) -> Self {
        let factor_sources_to_use = matrix_of_factor_sources.all_factors();
        let factor_sources = profile.factor_sources.clone();
        assert!(
            factor_sources.is_superset(&factor_sources_to_use),
            "Missing FactorSources"
        );
        assert!(!accounts.is_empty(), "No accounts");
        assert!(
            accounts.iter().all(|a| profile.contains_account(a.clone())),
            "unknown account"
        );
        let network_id = accounts.first().unwrap().network_id();
        assert!(
            accounts.iter().all(|a| a.network_id() == network_id),
            "wrong network"
        );

        let entity_kind = CAP26EntityKind::Account;
        let key_kind = CAP26KeyKind::TransactionSigning;
        let key_space = KeySpace::Securified;

        let agnostic_path = NetworkIndexAgnosticPath {
            entity_kind,
            key_kind,
            key_space,
        };

        Self::with(
            network_id,
            cache,
            factor_sources_to_use
                .into_iter()
                .map(|f| {
                    (
                        f.factor_source_id(),
                        QuantifiedNetworkIndexAgnosticPath {
                            quantity: accounts.len(),
                            agnostic_path,
                        },
                    )
                })
                .collect(),
        )
    }

    /// Supports loading many account vecis OR account mfa OR identity vecis OR identity mfa
    /// at once, does NOT support loading a mix of these. We COULD, but that would
    /// make the code more complex and we don't need it.
    fn with(
        network_id: NetworkID,
        cache: &mut Cache,
        index_agnostic_path_and_quantity_per_factor_source: IndexMap<
            FactorSourceIDFromHash,
            QuantifiedNetworkIndexAgnosticPath,
        >,
    ) -> Self {
        // `pf` is short for `Per FactorSource`
        let mut pf_found_in_cache = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();
        let factor_source_ids = index_agnostic_path_and_quantity_per_factor_source
            .keys()
            .cloned()
            .collect::<IndexSet<_>>();

        // For every factor source found in this map, we derive the remaining
        // quantity as to satisfy the request PLUS we are deriving to fill the
        // cache since we are deriving anyway, i.e. derive for all `IndexAgnosticPath`
        // "presets" (Account Veci, Identity Veci, Account MFA, Identity MFA).
        let mut pf_quantity_remaining_not_satisfied_by_cache =
            IndexMap::<FactorSourceIDFromHash, QuantifiedNetworkIndexAgnosticPath>::new();

        for (factor_source_id, quantified_agnostic_path) in
            index_agnostic_path_and_quantity_per_factor_source.iter()
        {
            let mut from_cache = FactorInstances::default();
            let mut unsatisfied_quantity = 0;
            let cache_key =
                IndexAgnosticPath::from((network_id, quantified_agnostic_path.agnostic_path));
            let quantity = quantified_agnostic_path.quantity;
            match cache.remove(&factor_source_id, &cache_key, quantity) {
                QuantityOutcome::Empty => {
                    from_cache = FactorInstances::default();
                    unsatisfied_quantity = quantity;
                }
                QuantityOutcome::Partial {
                    instances,
                    remaining,
                } => {
                    from_cache = instances;
                    unsatisfied_quantity = remaining;
                }
                QuantityOutcome::Full { instances } => {
                    from_cache = instances;
                    unsatisfied_quantity = 0;
                }
            }
            if unsatisfied_quantity > 0 {
                pf_quantity_remaining_not_satisfied_by_cache.insert(
                    factor_source_id.clone(),
                    QuantifiedNetworkIndexAgnosticPath {
                        quantity: unsatisfied_quantity,
                        agnostic_path: quantified_agnostic_path.agnostic_path,
                    },
                );
            }
            if !from_cache.is_empty() {
                pf_found_in_cache.insert(factor_source_id.clone(), from_cache.clone());
            }
        }

        let mut agnostic_paths_for_derivation_with_quantity_placeholders = IndexMap::<
            FactorSourceIDFromHash,
            IndexSet<CacheFillingPlaceholderQuantityNetworkIndexAgnosticPath>,
        >::new();

        for factor_source_id in factor_source_ids.iter() {
            let partial = pf_quantity_remaining_not_satisfied_by_cache
                .get(factor_source_id)
                .cloned();
            for preset in NetworkIndexAgnosticPath::all_presets() {
                let to_insert = partial
                    .and_then(|p| {
                        if p.agnostic_path == preset {
                            Some(
                                CacheFillingPlaceholderQuantityNetworkIndexAgnosticPath {
                                    quantity: CacheFillingPlaceholderQuantity::ToFulfillRequestAlsoCacheFilling { quantity_remaining_not_satisfied_by_cache: p.quantity },
                                    agnostic_path: p.agnostic_path,
                                }
                            )
                        } else {
                            None
                        }
                    })
                    .unwrap_or(CacheFillingPlaceholderQuantityNetworkIndexAgnosticPath {
                        quantity: CacheFillingPlaceholderQuantity::OnlyCacheFilling,
                        agnostic_path: preset,
                    });

                if let Some(existing) = agnostic_paths_for_derivation_with_quantity_placeholders
                    .get_mut(factor_source_id)
                {
                    existing.insert(to_insert);
                } else {
                    agnostic_paths_for_derivation_with_quantity_placeholders
                        .insert(factor_source_id.clone(), IndexSet::just(to_insert));
                }
            }
        }

        let paths = IndexMap::<FactorSourceIDFromHash, IndexSet<DerivationPath>>::new();

        let mut pf_to_cache = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();
        let mut pf_to_use_directly = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();
        let mut pf_newly_derived = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();

        // let keys_collector = KeysCollector::new(
        //     index_agnostic_path_and_quantity_per_factor_source
        //         .iter()
        //         .map(|(f, _)| f.clone())
        //         .collect(),
        //     paths,
        // );

        todo!()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_account_veci_derive_more_derives_identities() {}
}
