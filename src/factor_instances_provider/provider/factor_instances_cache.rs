use std::{borrow::Borrow, f32::consts::E, ops::Add, process::exit};

use crate::prelude::*;

/// A cache of factor instances.
///
/// Keyed under FactorSourceID and then under `IndexAgnosticPath`, each holding
/// an ordered set of Factor Instances, with contiguous derivation entity indices,
/// with lowest indices first in the set and highest last.
///
/// Since an IndexAgnosticPath essentially is the tuple `(NetworkID, DerivationPreset)`,
/// you can think of the implementation to not be:
/// `IndexMap<FactorSourceIDFromHash, IndexMap<IndexAgnosticPath, FactorInstances>>`
/// but actually:
/// IndexMap<FactorSourceIDFromHash, IndexMap<NetworkID, IndexMap<DerivationPreset, FactorInstances>>>`,
/// in fact it could be, but not sure it is more readable. But for the sake of visualizing
/// the cache we use that structure.
///
/// E.g.:
/// ```ignore
/// [
///     "FactorSourceID<Ledger3>": [
///         "Mainnet": [
///             DerivationPreset::AccountVeci: [
///                 (0', key...),
///                 (1', key...),
///                 ...
///                 (29', key...),
///             ],
///             DerivationPreset::AccountMfa: [
///                 (0^, key...),
///                 (1^, key...),
///                 ...
///                 (29^, key...),
///             ],
///            DerivationPreset::IdentityVeci: [
///                 (0', key...),
///                 ...
///                 (29', key...),
///             ],
///             DerivationPreset::IdentityMfa: [
///                 (0^, key...), ..., (29^, key...),
///             ],
///         ],
///         "Stokenet": [
///             DerivationPreset::AccountVeci: [
///                 (0', key...), ..., (29', key...),
///             ],
///             DerivationPreset::AccountMfa: [
///                 (0^, key...), ..., (29^, key...),
///             ],
///            DerivationPreset::IdentityVeci: [
///                 (0', key...), ... (29', key...),
///             ],
///             DerivationPreset::IdentityMfa: [
///                 (0^, key...), ..., (29^, key...),
///             ],
///         ],
///     ],
///     "FactorSourceID<Arculus5>": [
///         "Mainnet": [
///             DerivationPreset::AccountVeci: [
///                 (0', key...),  ...,  (29', key...),
///             ],
///             DerivationPreset::AccountMfa: [ ... ],
///             DerivationPreset::IdentityVeci: [ ... ],
///             DerivationPreset::IdentityMfa: [ ...  ],
///         ],
///         "Stokenet": [
///             DerivationPreset::AccountVeci: [
///                 (0', key...), ..., (29', key...),
///             ],
///             DerivationPreset::AccountMfa: [ ... ],
///             DerivationPreset::IdentityVeci: [ ... ],
///             DerivationPreset::IdentityMfa: [ ... ],
///         ],
///     ],
/// ]
/// ```
///
/// This is the "in-memory" form of the cache. We would need to impl `Serde` for
/// it in Sargon.
///
/// We use `IndexMap` instead of `HashMap` for future proofing when we serialize,
/// deserialize this cache, we want the JSON values to have stable ordering. Note
/// that the only truly **important** ordering is that of `FactorInstances` values,
/// which are ordered since it is a newtype around `IndexSet<HierarchicalDeterministicFactorInstance>`.
///
///
/// The Serde impl of `IndexAgnosticPath` could be:
/// `"<Network>/<EntityKind>/<KeyKind>/<KeySpace>"` as a string, e.g:
/// `"1/A/TX/U"`, where `U` is "Unsecurified" KeySpace.
/// Or if we don't wanna use such a "custom" one we can use `525`/`616`
/// discriminator for EntityKind and `1460`/`1678` for KeyKind:
/// "1/525/1460/U".
#[derive(Debug, Default, Clone)]
pub struct FactorInstancesCache {
    /// PER FactorSource PER IndexAgnosticPath FactorInstances (matching that IndexAgnosticPath)
    pub values: IndexMap<FactorSourceIDFromHash, IndexMap<IndexAgnosticPath, FactorInstances>>,
}

impl FactorInstancesCache {
    /// Inserts `instances` under `factor_source_id` by splitting them and grouping
    /// them by their `IndexAgnosticPath`.
    ///
    /// Returns `Err` if any of the instances is in fact does NOT have `factor_source_id`,
    /// as its factor source id.
    ///
    /// Returns `bool` indicating if an index was skipped resulting in non-contiguousness, which
    /// we do not use for now. Might be something we enforce or not for certain operations
    /// in the future.
    pub fn insert_for_factor(
        &mut self,
        factor_source_id: FactorSourceIDFromHash,
        instances: FactorInstances,
    ) -> Result<bool> {
        let instances = instances.into_iter().collect_vec();

        let instances_by_agnostic_path = instances
            .into_iter()
            .into_group_map_by(|f| f.agnostic_path())
            .into_iter()
            .map(|(k, v)| {
                if v.iter().any(|f| f.factor_source_id != factor_source_id) {
                    return Err(CommonError::FactorSourceDiscrepancy);
                }
                Ok((k, FactorInstances::from_iter(v)))
            })
            .collect::<Result<IndexMap<IndexAgnosticPath, FactorInstances>>>()?;
        let mut skipped_an_index_resulting_in_non_contiguousness = false;
        if let Some(existing_for_factor) = self.values.get_mut(&factor_source_id) {
            for (agnostic_path, instances) in instances_by_agnostic_path {
                let instances = instances.factor_instances();

                if let Some(existing_for_path) = existing_for_factor.get_mut(&agnostic_path) {
                    if let Some(fi) = instances
                        .intersection(&existing_for_path.factor_instances())
                        .next()
                    {
                        return Err(CommonError::CacheAlreadyContainsFactorInstance {
                            derivation_path: fi.derivation_path(),
                        });
                    }

                    if let Some(last) = existing_for_path.factor_instances().last() {
                        if instances.first().unwrap().derivation_entity_base_index()
                            != last.derivation_entity_base_index() + 1
                        {
                            warn!(
                                "Non-contiguous indices, the index `{}` was skipped!",
                                last.derivation_entity_base_index() + 1
                            );
                            skipped_an_index_resulting_in_non_contiguousness = true;
                        }
                    }
                    existing_for_path.extend(instances);
                } else {
                    existing_for_factor.insert(agnostic_path, FactorInstances::from(instances));
                }
            }
        } else {
            self.values
                .insert(factor_source_id, instances_by_agnostic_path);
        }

        Ok(skipped_an_index_resulting_in_non_contiguousness)
    }

    /// Inserts all instance in `per_factor`.
    pub fn insert_all(
        &mut self,
        per_factor: IndexMap<FactorSourceIDFromHash, FactorInstances>,
    ) -> Result<()> {
        for (factor_source_id, instances) in per_factor {
            _ = self.insert_for_factor(factor_source_id, instances)?;
        }
        Ok(())
    }

    pub fn max_index_for(
        &self,
        factor_source_id: FactorSourceIDFromHash,
        agnostic_path: IndexAgnosticPath,
    ) -> Option<HDPathComponent> {
        let Some(for_factor) = self.values.get(&factor_source_id) else {
            return None;
        };
        let Some(instances) = for_factor.get(&agnostic_path) else {
            return None;
        };
        instances
            .factor_instances()
            .last()
            .map(|fi| fi.derivation_entity_index())
    }

    pub fn get_poly_factor(
        &self,
        factor_source_ids: &IndexSet<FactorSourceIDFromHash>,
        index_agnostic_path: &IndexAgnosticPath,
    ) -> Result<IndexMap<FactorSourceIDFromHash, FactorInstances>> {
        let mut pf = IndexMap::new();
        for factor_source_id in factor_source_ids {
            let Some(instances) = self.get_mono_factor(factor_source_id, index_agnostic_path)
            else {
                continue;
            };
            pf.insert(factor_source_id.clone(), instances);
        }
        Ok(pf)
    }

    pub fn get_poly_factor_with_quantities(
        &self,
        factor_source_ids: &IndexSet<FactorSourceIDFromHash>,
        originally_requested_quantified_derivation_preset: &QuantifiedDerivationPresets,
        network_id: NetworkID,
    ) -> Result<CachedInstancesWithQuantities> {
        todo!()
    }
}

#[derive(enum_as_inner::EnumAsInner)]
enum CachedInstancesWithQuantitiesOutcome {
    Satisfied(IndexMap<FactorSourceIDFromHash, FactorInstances>),
    NotSatisfied(IndexMap<FactorSourceIDFromHash, FactorInstances>),
}
pub struct CachedInstancesWithQuantities {
    originally_requested_quantified_derivation_preset: QuantifiedDerivationPresets,
    network_id: NetworkID,
    outcome: CachedInstancesWithQuantitiesOutcome,
}
impl CachedInstancesWithQuantities {
    pub fn satisfied(&self) -> Option<IndexMap<FactorSourceIDFromHash, FactorInstances>> {
        self.outcome.as_satisfied().cloned()
    }
    pub fn quantities_to_derive(
        &self,
    ) -> IndexMap<FactorSourceIDFromHash, IndexMap<DerivationPreset, usize>> {
        let instances = self._not_requested();
        todo!()
    }
    fn _not_requested(&self) -> IndexMap<FactorSourceIDFromHash, FactorInstances> {
        self.outcome
            .as_not_satisfied()
            .cloned()
            .expect("not satisfied")
    }
    pub fn get_requested(self) -> IndexMap<FactorSourceIDFromHash, FactorInstances> {
        self._not_requested()
    }
}

impl FactorInstancesCache {
    pub fn get_mono_factor(
        &self,
        factor_source_id: &FactorSourceIDFromHash,
        index_agnostic_path: &IndexAgnosticPath,
    ) -> Option<FactorInstances> {
        let Some(for_factor) = self.values.get(factor_source_id) else {
            return None;
        };
        let Some(instances) = for_factor.get(index_agnostic_path) else {
            return None;
        };
        Some(instances.clone())
    }

    pub fn delete(&mut self, pf_instances: IndexMap<FactorSourceIDFromHash, FactorInstances>) {
        for (factor_source_id, instances_to_delete) in pf_instances {
            if instances_to_delete.is_empty() {
                continue;
            }
            let existing_for_factor = self
                .values
                .get_mut(&factor_source_id)
                .expect("expected to delete factors");

            let instances_to_delete_by_path = instances_to_delete.factor_instances()
                .into_iter()
                .into_group_map_by(|f| {
                    f.agnostic_path()
                })
                .into_iter()
                .collect::<IndexMap<IndexAgnosticPath, Vec<HierarchicalDeterministicFactorInstance>>>();

            for (index_agnostic_path, instances_to_delete) in instances_to_delete_by_path {
                let instances_to_delete =
                    IndexSet::<HierarchicalDeterministicFactorInstance>::from_iter(
                        instances_to_delete.into_iter(),
                    );

                let existing_for_path = existing_for_factor
                    .get(&index_agnostic_path)
                    .expect("expected to delete")
                    .factor_instances();

                if !existing_for_path.is_superset(&instances_to_delete) {
                    panic!("Programmer error! Some of the factors to delete were not in cache!");
                }
                let to_keep = existing_for_path
                    .symmetric_difference(&instances_to_delete)
                    .cloned()
                    .collect::<FactorInstances>();

                // replace
                existing_for_factor.insert(index_agnostic_path, to_keep);
            }
        }
    }

    pub fn insert(&mut self, pf_instances: IndexMap<FactorSourceIDFromHash, FactorInstances>) {
        self.insert_all(pf_instances).expect("works")
    }

    /// Reads out the instance of `factor_source_id` without mutating the cache.
    pub fn peek_all_instances_of_factor_source(
        &self,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Option<IndexMap<IndexAgnosticPath, FactorInstances>> {
        self.values.get(&factor_source_id).cloned()
    }

    #[cfg(test)]
    pub fn total_number_of_factor_instances(&self) -> usize {
        self.values
            .values()
            .map(|x| {
                x.values()
                    .map(|y| y.len())
                    .reduce(Add::add)
                    .unwrap_or_default()
            })
            .reduce(Add::add)
            .unwrap_or_default()
    }
}

/// The outcome of reading factor instances from the cache, for a requested
/// quantity.
pub enum QuantityOutcome {
    /// No `FactorInstances` was found for the request `(FactorSourceID, IndexAgnosticPath, Quantity)`
    Empty,

    /// Only some `FactorInstances` was found for the request `(FactorSourceID, IndexAgnosticPath, Quantity)`,
    /// being less than the requested quantity
    Partial {
        /// (NonEmpty) Instances found in cache, which is fewer than `originally_requested`
        instances: FactorInstances,
        /// Remaining quantity to satisfy the request, `originally_requested - instances.len()`
        remaining: usize,
    },

    /// The cache contained enough `FactorInstances` for the request `(FactorSourceID, IndexAgnosticPath, Quantity)`,
    /// either the exact same amount, or with "spare" ones.
    Full {
        /// (NonEmpty) Instances found in cache, which has the same length as `originally_requested`
        instances: FactorInstances,
    },
}

impl FactorInstancesCache {
    /// Removes all FactorInstances matching (FactorSourceID, IndexAgnosticPath),
    /// and returns them - if any.
    fn __remove(
        &mut self,
        factor_source_id: &FactorSourceIDFromHash,
        index_agnostic_path: &IndexAgnosticPath,
    ) -> FactorInstances {
        if let Some(cached_for_factor) = self.values.get_mut(factor_source_id) {
            if let Some(found_cached) = cached_for_factor.shift_remove(index_agnostic_path) {
                return found_cached;
            }
        }
        FactorInstances::default()
    }

    /// Mutates the cache, removing `quantity` many FactorInstances for `factor_source_id`
    /// for `index_agnostic_path` and return the outcome of this, this result in any
    /// of these outcomes:
    /// * Empty
    /// * Partial
    /// * Full
    pub fn remove(
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
        // Split the read instances at `quantity` many.
        let instances = instances.factor_instances().into_iter().collect_vec();
        let (to_use, to_put_back) = instances.split_at(quantity);
        let to_put_back = FactorInstances::from_iter(to_put_back.iter().cloned());

        // put back the ones exceeding requested quantity
        if let Some(cached_for_factor) = self.values.get_mut(factor_source_id) {
            cached_for_factor.insert(*index_agnostic_path, to_put_back);
        }

        QuantityOutcome::Full {
            instances: FactorInstances::from_iter(to_use.iter().cloned()),
        }
    }
}

#[cfg(test)]
impl FactorInstancesCache {
    /// Queries the cache to see if the cache is full for factor_source_id for
    /// each DerivationPreset
    pub fn is_full(&self, network_id: NetworkID, factor_source_id: FactorSourceIDFromHash) -> bool {
        let count: usize = self
            .values
            .get(&factor_source_id)
            .and_then(|c| {
                c.values()
                    .map(|xs| {
                        xs.factor_instances()
                            .iter()
                            .filter(|x| x.agnostic_path().network_id == network_id)
                            .collect_vec()
                            .len()
                    })
                    .reduce(Add::add)
            })
            .unwrap_or(0);

        count == DerivationPreset::all().len() * CACHE_FILLING_QUANTITY
    }

    pub fn assert_is_full(&self, network_id: NetworkID, factor_source_id: FactorSourceIDFromHash) {
        assert!(self.is_full(network_id, factor_source_id));
    }
}

#[cfg(test)]
mod tests {

    use std::fs;

    use super::*;

    type Sut = FactorInstancesCache;

    #[test]
    fn non_contiguous_indices() {
        let mut sut = Sut::default();
        let fsid = FactorSourceIDFromHash::fs0();
        let fi0 = HierarchicalDeterministicFactorInstance::mainnet_tx(
            CAP26EntityKind::Account,
            HDPathComponent::unsecurified_hardening_base_index(0),
            fsid,
        );
        assert!(!sut
            .insert_for_factor(fsid, FactorInstances::from_iter([fi0]))
            .unwrap());
        let fi2 = HierarchicalDeterministicFactorInstance::mainnet_tx(
            CAP26EntityKind::Account,
            HDPathComponent::unsecurified_hardening_base_index(2), // OH NO! Skipping `1`
            fsid,
        );
        assert!(sut
            .insert_for_factor(fsid, FactorInstances::from_iter([fi2]))
            .unwrap(),);
    }

    #[test]
    fn factor_source_discrepancy() {
        let mut sut = Sut::default();
        let fi0 = HierarchicalDeterministicFactorInstance::mainnet_tx(
            CAP26EntityKind::Account,
            HDPathComponent::unsecurified_hardening_base_index(0),
            FactorSourceIDFromHash::fs0(),
        );
        assert!(sut
            .insert_for_factor(
                FactorSourceIDFromHash::fs1(),
                FactorInstances::from_iter([fi0])
            )
            .is_err());
    }

    #[test]
    fn delete() {
        let mut sut = Sut::default();

        let factor_source_ids = HDFactorSource::all()
            .into_iter()
            .map(|f| f.factor_source_id())
            .collect::<IndexSet<_>>();

        let n = 30;
        let mut to_delete = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();
        let mut to_remain = IndexMap::<FactorSourceIDFromHash, FactorInstances>::new();
        for factor_source_id in factor_source_ids.clone() {
            let fsid = factor_source_id;
            let instances = (0..n)
                .map(|i| {
                    let fi = HierarchicalDeterministicFactorInstance::mainnet_tx(
                        CAP26EntityKind::Account,
                        HDPathComponent::unsecurified_hardening_base_index(i),
                        fsid,
                    );
                    if i < 10 {
                        to_delete.append_or_insert_to(&fsid, IndexSet::just(fi.clone()));
                    } else {
                        to_remain.append_or_insert_to(&fsid, IndexSet::just(fi.clone()));
                    }
                    fi
                })
                .collect::<IndexSet<_>>();

            sut.insert_for_factor(fsid, FactorInstances::from(instances))
                .unwrap();
        }

        sut.delete(to_delete);
        assert_eq!(
            sut.get_poly_factor(
                &factor_source_ids,
                &IndexAgnosticPath::new(
                    NetworkID::Mainnet,
                    CAP26EntityKind::Account,
                    CAP26KeyKind::TransactionSigning,
                    KeySpace::Unsecurified
                )
            )
            .unwrap(),
            to_remain
        );
    }

    #[test]
    fn throws_if_same_is_added() {
        let mut sut = Sut::default();
        let fsid = FactorSourceIDFromHash::fs0();
        let fi0 = HierarchicalDeterministicFactorInstance::mainnet_tx(
            CAP26EntityKind::Account,
            HDPathComponent::unsecurified_hardening_base_index(0),
            fsid,
        );
        let fi1 = HierarchicalDeterministicFactorInstance::mainnet_tx(
            CAP26EntityKind::Account,
            HDPathComponent::unsecurified_hardening_base_index(1),
            fsid,
        );
        assert!(!sut
            .insert_for_factor(fsid, FactorInstances::from_iter([fi0.clone(), fi1]))
            .unwrap());

        assert_eq!(
            sut.insert_for_factor(fsid, FactorInstances::from_iter([fi0.clone()]))
                .err()
                .unwrap(),
            CommonError::CacheAlreadyContainsFactorInstance {
                derivation_path: fi0.derivation_path()
            }
        );
    }
}

pub trait AppendableCollection: FromIterator<Self::Element> {
    type Element: Eq + std::hash::Hash;
    fn append<T: IntoIterator<Item = Self::Element>>(&mut self, iter: T);
}
impl<V: Eq + std::hash::Hash> AppendableCollection for IndexSet<V> {
    type Element = V;

    fn append<T: IntoIterator<Item = Self::Element>>(&mut self, iter: T) {
        self.extend(iter)
    }
}

impl AppendableCollection for FactorInstances {
    type Element = HierarchicalDeterministicFactorInstance;

    fn append<T: IntoIterator<Item = Self::Element>>(&mut self, iter: T) {
        self.extend(iter)
    }
}

pub trait AppendableMap {
    type Key: Eq + std::hash::Hash + Clone;
    type AC: AppendableCollection;
    fn append_or_insert_to<I: IntoIterator<Item = <Self::AC as AppendableCollection>::Element>>(
        &mut self,
        key: impl Borrow<Self::Key>,
        items: I,
    );

    fn append_or_insert_element_to(
        &mut self,
        key: impl Borrow<Self::Key>,
        element: <Self::AC as AppendableCollection>::Element,
    ) {
        self.append_or_insert_to(key.borrow(), [element]);
    }
}

impl<K, V> AppendableMap for IndexMap<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: AppendableCollection,
{
    type Key = K;
    type AC = V;
    fn append_or_insert_to<I: IntoIterator<Item = <Self::AC as AppendableCollection>::Element>>(
        &mut self,
        key: impl Borrow<Self::Key>,
        items: I,
    ) {
        let key = key.borrow();
        if let Some(existing) = self.get_mut(key) {
            existing.append(items);
        } else {
            self.insert(key.clone(), V::from_iter(items));
        }
    }
}

#[cfg(test)]
mod test_appendable_collection {
    use super::*;

    #[test]
    fn test_append_element() {
        type Sut = IndexMap<i8, IndexSet<u8>>;
        let mut map = Sut::new();
        map.append_or_insert_element_to(-3, 5);
        map.append_or_insert_element_to(-3, 6);
        map.append_or_insert_element_to(-3, 6);
        map.append_or_insert_to(-3, [42, 237]);
        map.append_or_insert_to(-9, [64, 128]);
        assert_eq!(
            map,
            Sut::from_iter([
                (-3, IndexSet::<u8>::from_iter([5, 6, 42, 237, 237])),
                (-9, IndexSet::<u8>::from_iter([64, 128])),
            ])
        );
    }
}
