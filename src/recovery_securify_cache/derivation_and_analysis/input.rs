#![allow(unused)]

use std::ops::Index;

use crate::prelude::*;

pub struct ProfileNextIndexAnalyzer {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntermediaryDerivationAnalysis {
    all_factor_instances: Vec<HierarchicalDeterministicFactorInstance>,
    pub probably_free_instances: ProbablyFreeFactorInstances,
    pub recovered_unsecurified_entities: RecoveredUnsecurifiedEntities,
    pub recovered_securified_entities: RecoveredSecurifiedEntities,
    pub unrecovered_securified_entities: UnrecoveredSecurifiedEntities,
}
impl IntermediaryDerivationAnalysis {
    /// # Panics
    /// Panics if the union all instances from:
    /// `probably_free_instances`, `recovered_unsecurified_entities`, `recovered_securified_entities`, `unrecovered_securified_entities` are not equal to `all_factor_instances`
    /// Also panics if any factor_instance in:
    /// `recovered_unsecurified_entities`, `recovered_securified_entities`, `unrecovered_securified_entities` is found in probably_free_instances
    pub fn new(
        all_factor_instances: IndexSet<HierarchicalDeterministicFactorInstance>,
        probably_free_instances: ProbablyFreeFactorInstances,
        recovered_unsecurified_entities: RecoveredUnsecurifiedEntities,
        recovered_securified_entities: RecoveredSecurifiedEntities,
        unrecovered_securified_entities: UnrecoveredSecurifiedEntities,
    ) -> Self {
        let mut merge = IndexSet::new();
        merge.extend(recovered_unsecurified_entities.instances());
        merge.extend(recovered_securified_entities.instances());
        merge.extend(unrecovered_securified_entities.instances());

        assert!(
            merge.clone().into_iter().collect::<HashSet<_>>()
            .intersection(
                &probably_free_instances.instances().into_iter().collect::<HashSet<_>>()
            )
            .collect_vec()
            .is_empty(),
            "Discrepancy! Some factor instances in probably_free_instances are found in other collections of non free instances!");

        // Only extend with `probably_free_instances` once we have verified it does not contain any of the instances in the other collections
        merge.extend(probably_free_instances.instances());

        assert_eq!(
            merge.into_iter().collect::<HashSet<_>>(),
            all_factor_instances
                .clone()
                .into_iter()
                .collect::<HashSet<_>>()
        );

        Self {
            all_factor_instances: all_factor_instances.into_iter().collect(),
            probably_free_instances,
            recovered_unsecurified_entities,
            recovered_securified_entities,
            unrecovered_securified_entities,
        }
    }
}

pub struct DerivationRequest {
    pub factor_source_id: FactorSourceIDFromHash,
    pub network_id: NetworkID,
    pub entity_kind: CAP26EntityKind,
    pub key_space: KeySpace,
    pub key_kind: CAP26KeyKind,
}

/// Lookup which Entity Range Start Value to use for the Range of Derivation Indices
/// to uses per request.
///
/// E.g. when doing recovery scan of unsecurified accounts we would return:
/// `HDPathComponent::unsecurified_hardening_base_index(0)` as start value of a range of lets say
/// 30 indices, for `DeviceFactorSource`s, given a request:
/// `DerivationRequest { entity_kind: Account, key_space: Unsecurified, key_kind: TransactionSigning, ... }`
///
/// But for VECID - Virtual Entity Creating (Factor)Instance Derivation request:
/// `DerivationRequest { entity_kind: Account, key_space: Unsecurified, key_kind: TransactionSigning, ... }`
/// we would use "next free" based on cache or profile analysis.
pub struct NextDerivationEntityIndexAssigner {
    start_index: Arc<dyn Fn(DerivationRequest) -> HDPathComponent>,
}

/// Check if there is any known entity associated with a given factor instance,
/// if so, some base info, if not, it is counted as "probably free".
pub struct IntermediaryDerivationAnalyzer {
    analuze: Arc<
        dyn Fn(IndexSet<HierarchicalDeterministicFactorInstance>) -> IntermediaryDerivationAnalysis,
    >,
}

pub struct DeriveAndAnalyzeInput {
    factor_sources: IndexSet<HDFactorSource>,
    ids_of_new_factor_sources: IndexSet<FactorSourceIDFromHash>,

    /// Which index to start at for the range of indices to derive
    next_derivation_entity_index_assigner: NextDerivationEntityIndexAssigner,

    /// Check if there is any known entity associated with a given factor instance,
    /// if so, some base info, if not, it is counted as "probably free".
    analyze_factor_instances: IntermediaryDerivationAnalyzer,
}

impl DeriveAndAnalyzeInput {
    pub fn all_factor_sources(&self) -> IndexSet<HDFactorSource> {
        self.factor_sources.clone().into_iter().collect()
    }
    pub fn new_factor_sources(&self) -> IndexSet<HDFactorSource> {
        self.all_factor_sources()
            .into_iter()
            .filter(|f| {
                !self
                    .ids_of_new_factor_sources
                    .contains(&f.factor_source_id())
            })
            .collect()
    }
    pub fn old_factor_sources(&self) -> IndexSet<HDFactorSource> {
        self.all_factor_sources()
            .into_iter()
            .filter(|f| {
                self.ids_of_new_factor_sources
                    .contains(&f.factor_source_id())
            })
            .collect()
    }
}

impl DeriveAndAnalyzeInput {
    /// # Panics
    /// Panics if some IDs of `ids_of_new_factor_sources` are not found in `factor_sources`
    pub fn new(
        factor_sources: IndexSet<HDFactorSource>,
        ids_of_new_factor_sources: IndexSet<FactorSourceIDFromHash>,
        next_derivation_entity_index_assigner: NextDerivationEntityIndexAssigner,
        analyze_factor_instances: IntermediaryDerivationAnalyzer,
    ) -> Self {
        assert!(
            ids_of_new_factor_sources
                .iter()
                .all(|id| factor_sources.iter().any(|f| f.factor_source_id() == *id)),
            "Discrepancy! Some IDs of new factor sources are not found in factor sources!"
        );

        Self {
            factor_sources,
            ids_of_new_factor_sources,
            next_derivation_entity_index_assigner,
            analyze_factor_instances,
        }
    }
}
