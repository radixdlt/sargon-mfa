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

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct QuantifiedNetworkIndexAgnosticPath {
    pub agnostic_path: NetworkIndexAgnosticPath,
    pub quantity: usize,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct QuantifiedToCacheToUseNetworkIndexAgnosticPath {
    pub agnostic_path: NetworkIndexAgnosticPath,
    pub quantity: QuantityToCacheToUseDirectly,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct QuantifiedToCacheToUseIndexAgnosticPath {
    pub agnostic_path: IndexAgnosticPath,
    pub quantity: QuantityToCacheToUseDirectly,
}

impl From<(IndexAgnosticPath, HDPathComponent)> for DerivationPath {
    fn from((path, index): (IndexAgnosticPath, HDPathComponent)) -> Self {
        assert_eq!(index.key_space(), path.key_space);
        Self::new(path.network_id, path.entity_kind, path.key_kind, index)
    }
}

pub const CACHE_FILLING_QUANTITY: usize = 30;
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum QuantityToCacheToUseDirectly {
    OnlyCacheFilling {
        /// Typically (always?) `CACHE_FILLING_QUANTITY`
        fill_cache: usize,
    },

    /// We will derive `remaining + extra_to_fill_cache` more instances
    ToCacheToUseDirectly {
        /// Remaining quantity to satisfy the request, `originally_requested - from_cache_instances.len()`
        /// Used later to split the newly derived instances into two groups, to cache and to use directly,
        /// can be zero.
        remaining: usize,

        /// Typically (always?) `CACHE_FILLING_QUANTITY`
        extra_to_fill_cache: usize,
    },
}
impl QuantityToCacheToUseDirectly {
    pub fn total_quantity_to_derive(&self) -> usize {
        match self {
            Self::OnlyCacheFilling { fill_cache } => *fill_cache,
            Self::ToCacheToUseDirectly {
                remaining,
                extra_to_fill_cache,
            } => *remaining + *extra_to_fill_cache,
        }
    }
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
        /// (NonEmpty) Instances found in cache, which is fewer than `originally_requested`
        instances: FactorInstances,
        /// Remaining quantity to satisfy the request, `originally_requested - instances.len()`
        remaining: usize,
    },
    Full {
        /// (NonEmpty) Instances found in cache, which has the same length as `originally_requested`
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
            cached_for_factor.insert(*index_agnostic_path, to_put_back);
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
impl NextIndexAssigner {
    pub fn next_index(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        agnostic_path: IndexAgnosticPath,
    ) -> HDPathComponent {
        todo!()
    }
}

pub struct FactorInstancesProvider {
    cache: Cache,
    next_index_assigner: NextIndexAssigner,
}

pub struct KeysCollector;
impl KeysCollector {
    fn new(
        _factor_sources: FactorSources,
        _paths_per_factor: IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>,
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
        next_index_assigner: &NextIndexAssigner,
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
            next_index_assigner,
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
        next_index_assigner: &NextIndexAssigner,
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
            match cache.remove(factor_source_id, &cache_key, quantity) {
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
                    *factor_source_id,
                    QuantifiedNetworkIndexAgnosticPath {
                        quantity: unsatisfied_quantity,
                        agnostic_path: quantified_agnostic_path.agnostic_path,
                    },
                );
            }
            if !from_cache.is_empty() {
                pf_found_in_cache.insert(*factor_source_id, from_cache.clone());
            }
        }

        let mut pf_quantified_network_agnostic_paths_for_derivation = IndexMap::<
            FactorSourceIDFromHash,
            IndexSet<QuantifiedToCacheToUseNetworkIndexAgnosticPath>,
        >::new();

        for factor_source_id in factor_source_ids.iter() {
            let partial = pf_quantity_remaining_not_satisfied_by_cache
                .get(factor_source_id)
                .cloned();
            for preset in NetworkIndexAgnosticPath::all_presets() {
                let to_insert = partial
                    .and_then(|p| {
                        if p.agnostic_path == preset {
                            Some(QuantifiedToCacheToUseNetworkIndexAgnosticPath {
                                quantity: QuantityToCacheToUseDirectly::ToCacheToUseDirectly {
                                    remaining: p.quantity,
                                    extra_to_fill_cache: CACHE_FILLING_QUANTITY,
                                },
                                agnostic_path: p.agnostic_path,
                            })
                        } else {
                            None
                        }
                    })
                    .unwrap_or(QuantifiedToCacheToUseNetworkIndexAgnosticPath {
                        quantity: QuantityToCacheToUseDirectly::OnlyCacheFilling {
                            fill_cache: CACHE_FILLING_QUANTITY,
                        },
                        agnostic_path: preset,
                    });

                if let Some(existing) =
                    pf_quantified_network_agnostic_paths_for_derivation.get_mut(factor_source_id)
                {
                    existing.insert(to_insert);
                } else {
                    pf_quantified_network_agnostic_paths_for_derivation
                        .insert(*factor_source_id, IndexSet::just(to_insert));
                }
            }
        }

        // Now map from NetworkAgnostic to NetworkAware paths, but still index agnostic
        let pf_quantified_index_agnostic_paths_for_derivation =
            pf_quantified_network_agnostic_paths_for_derivation
                .into_iter()
                .map(|(factor_source_id, quantified_network_agnostic_paths)| {
                    let index_agnostic_paths = quantified_network_agnostic_paths
                        .into_iter()
                        .map(|q| QuantifiedToCacheToUseIndexAgnosticPath {
                            agnostic_path: IndexAgnosticPath::from((network_id, q.agnostic_path)),
                            quantity: q.quantity,
                        })
                        .collect::<IndexSet<_>>();
                    (factor_source_id, index_agnostic_paths)
                })
                .collect::<IndexMap<_, _>>();

        // Now map from IndexAgnostic paths to index aware paths, a.k.a. DerivationPath
        // but ALSO we need to retain the information of how many factor instances of
        // the newly derived to append to the factor instances to use directly, and how many to cache.
        let paths = pf_quantified_index_agnostic_paths_for_derivation
            .clone()
            .into_iter()
            .map(|(f, agnostic_paths)| {
                let paths = agnostic_paths
                    .clone()
                    .into_iter()
                    .flat_map(|quantified_agnostic_path| {
                        // IMPORTANT! We are not mapping one `IndexAgnosticPath` to one `DerivationPath`, but
                        // rather we are mapping one `IndexAgnosticPath` to many `DerivationPath`! Equal to
                        // the same number as the specified quantity!
                        (0..quantified_agnostic_path.quantity.total_quantity_to_derive())
                            .map(|_| {
                                let index = next_index_assigner
                                    .next_index(f, quantified_agnostic_path.agnostic_path);
                                DerivationPath::from((
                                    quantified_agnostic_path.agnostic_path,
                                    index,
                                ))
                            })
                            .collect::<IndexSet<_>>()
                    })
                    .into_iter()
                    .collect::<IndexSet<_>>();
                (f, paths)
            })
            .collect::<IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>>();

        let _pf_to_cache = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();
        let _pf_to_use_directly = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();
        let _pf_newly_derived = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();

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
