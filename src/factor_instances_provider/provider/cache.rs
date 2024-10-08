use crate::prelude::*;

#[derive(Debug, Default, Clone)]
pub struct Cache {
    /// PER FactorSource PER IndexAgnosticPath some value T
    pub values: HashMap<FactorSourceIDFromHash, HashMap<IndexAgnosticPath, FactorInstances>>,
}

impl Cache {
    pub fn insert_for_factor(
        &mut self,
        factor_source_id: FactorSourceIDFromHash,
        instances: FactorInstances,
    ) {
        let instances = instances.into_iter().collect_vec();

        let instances_by_agnostic_path = instances
            .into_iter()
            .into_group_map_by(|f| f.agnostic_path())
            .into_iter()
            .map(|(k, v)| (k, FactorInstances::from_iter(v)))
            .collect::<HashMap<IndexAgnosticPath, FactorInstances>>();

        if let Some(existing_for_factor) = self.values.get_mut(&factor_source_id) {
            for (agnostic_path, instances) in instances_by_agnostic_path {
                if let Some(existing_for_path) = existing_for_factor.get_mut(&agnostic_path) {
                    if let Some(last) = existing_for_path.factor_instances().last() {
                        assert_eq!(
                            last.derivation_entity_base_index() + 1,
                            instances
                                .factor_instances()
                                .first()
                                .unwrap()
                                .derivation_entity_base_index(),
                            "non contiguous indices"
                        )
                    }
                    existing_for_path.extend(instances);
                } else {
                    existing_for_factor.insert(agnostic_path, instances);
                }
            }
        } else {
            self.values
                .insert(factor_source_id, instances_by_agnostic_path);
        }
    }

    pub fn insert_all(
        &mut self,
        per_factor: IndexMap<FactorSourceIDFromHash, FactorInstances>,
    ) -> Result<()> {
        for (factor_source_id, instances) in per_factor {
            self.insert_for_factor(factor_source_id, instances);
        }
        Ok(())
    }

    pub fn peek_all_instances_of_factor_source(
        &self,
        factor_source_id: FactorSourceIDFromHash,
    ) -> Option<HashMap<IndexAgnosticPath, FactorInstances>> {
        self.values.get(&factor_source_id).cloned()
    }
}

pub enum QuantityOutcome {
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
