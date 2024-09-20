#![allow(unused)]

use std::ops::Index;

use crate::prelude::*;

pub struct ProfileNextIndexAnalyzer {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntermediaryDerivationAnalysis {
    pub probably_free_instances: ProbablyFreeFactorInstances,
    pub known_taken: KnownTakenInstances,
}

impl IntermediaryDerivationAnalysis {
    /// # Panics
    /// Panics if the collections of factor instances are not disjoint
    pub fn new(
        probably_free_instances: ProbablyFreeFactorInstances,
        known_taken: KnownTakenInstances,
    ) -> Self {
        assert_are_factor_instance_collections_disjoint(vec![
            &probably_free_instances,
            &known_taken,
        ]);

        Self {
            probably_free_instances,
            known_taken,
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
#[async_trait::async_trait]
pub trait IsNextDerivationEntityIndexAssigner {
    async fn start_index(&self, derivation_request: DerivationRequest) -> Result<HDPathComponent>;
}

/// Check if there is any known entity associated with a given factor instance,
/// if so, some base info, if not, it is counted as "probably free".
#[async_trait::async_trait]
pub trait IsIntermediaryDerivationAnalyzer {
    async fn analyze(
        &self,
        factor_instances: IndexSet<HierarchicalDeterministicFactorInstance>,
    ) -> Result<IntermediaryDerivationAnalysis>;
}

pub struct DeriveAndAnalyzeInput {
    factor_sources: IndexSet<HDFactorSource>,
    ids_of_new_factor_sources: IndexSet<FactorSourceIDFromHash>,

    /// Which index to start at for the range of indices to derive
    next_derivation_entity_index_assigner: Arc<dyn IsNextDerivationEntityIndexAssigner>,

    /// Check if there is any known entity associated with a given factor instance,
    /// if so, some base info, if not, it is counted as "probably free".
    pub analyze_factor_instances: Arc<dyn IsIntermediaryDerivationAnalyzer>,
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
        next_derivation_entity_index_assigner: Arc<dyn IsNextDerivationEntityIndexAssigner>,
        analyze_factor_instances: Arc<dyn IsIntermediaryDerivationAnalyzer>,
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
