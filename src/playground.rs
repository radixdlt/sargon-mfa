#![allow(unused)]

use std::ops::{Deref, Index};

use crate::prelude::*;

enum ScanHookDecision {
    /// "Probably" since we might not have all the information to be sure, since
    /// Gateway might not keep track of past FactorInstances, some of the FactorInstances
    /// in KeySpace::Securified might in fact have been used in the past for some entity.
    ProbablyIsFree(HierarchicalDeterministicFactorInstance),
    UnsecurifiedEntityRecovered {
        unsecurified_entity: AccountOrPersona,
        factor_instance: HierarchicalDeterministicFactorInstance,
    },
    SecurifiedEntityReferencesFactor {
        entity: SecurifiedEntity,
        factor_instance: HierarchicalDeterministicFactorInstance,
    },
}

enum ScanHookDecisionForGateway {
    /// "Probably" since we might not have all the information to be sure, since
    /// Gateway might not keep track of past FactorInstances, some of the FactorInstances
    /// in KeySpace::Securified might in fact have been used in the past for some entity.
    ProbablyIsFree(HierarchicalDeterministicFactorInstance),
    UnsecurifiedEntityRecovered {
        unsecurified_entity: OnChainEntityUnsecurified,
        factor_instance: HierarchicalDeterministicFactorInstance,
    },
    SecurifiedEntityReferencesFactor {
        securified_entity: OnChainEntitySecurified,
        factor_instance: HierarchicalDeterministicFactorInstance,
    },
}

type ScanHookSync<'a> = Box<
    dyn FnOnce(
            IndexSet<HierarchicalDeterministicFactorInstance>,
        ) -> IndexMap<HierarchicalDeterministicFactorInstance, ScanHookDecision>
        + 'a,
>;

type ScanHookGateway<'a> = Box<
    dyn Fn(
        IndexSet<HierarchicalDeterministicFactorInstance>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        IndexMap<
                            HierarchicalDeterministicFactorInstance,
                            ScanHookDecisionForGateway,
                        >,
                    >,
                > + 'a,
        >,
    >,
>;

pub struct FactorSourceDerivations {
    derivations_per_factor_source: IndexMap<HDFactorSource, HDPathValue>,
}
impl FactorSourceDerivations {
    pub fn single_factor_source(
        factor_source: HDFactorSource,
        start_base_index_for_each_key_space: HDPathValue,
    ) -> Self {
        Self {
            derivations_per_factor_source: IndexMap::just((
                factor_source,
                start_base_index_for_each_key_space,
            )),
        }
    }
    pub fn add_factor_source(factor_source: HDFactorSource) -> Self {
        Self::single_factor_source(factor_source, 0)
    }
    pub fn recovery(factor_sources: IndexSet<HDFactorSource>) -> Self {
        Self {
            derivations_per_factor_source: factor_sources
                .into_iter()
                .map(|factor_source| (factor_source, 0))
                .collect(),
        }
    }
}

async fn scan(
    factor_source_derivations: FactorSourceDerivations,
    profile_scan_hook: Option<ScanHookSync<'_>>,
    gateway: Arc<dyn GatewayReadonly>,
) -> Result<(IndexSet<AccountOrPersona>, ProbablyFreeFactorInstances)> {
    todo!()
}

impl Profile {
    fn add_all_entities(&mut self, entities: IndexSet<AccountOrPersona>) {
        todo!()
    }

    pub async fn new_unsecurified_entity<E: IsEntity>(
        &mut self,
        name: String,
        network_id: NetworkID,
        factor_source: HDFactorSource,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        gateway: Arc<dyn Gateway>,
        cache: Arc<dyn IsPreDerivedKeysCache>,
    ) -> Result<E> {
        let entity_kind = E::kind();
        let factor_source_id = factor_source.factor_source_id();
        let request = DerivationRequest::virtual_entity_creating_factor_instance(
            entity_kind,
            factor_source_id,
            network_id,
        );
        let can_consume_next_cache = cache.can_consume_next_factor(request).await;
        if can_consume_next_cache {
            let instance = cache.consume_next_factor_instance(request).await?;
        } else {
            // 🔶 🔶 🔶 🔶 🔶 🔶 🔶 🔶 🔶 🔶 🔶 🔶 🔶 🔶 🔶
            //  FILL CACHE! WITH INDEX OFFSETS
            // 🔶 🔶 🔶 🔶 🔶 🔶 🔶 🔶 🔶 🔶 🔶 🔶 🔶 🔶 🔶
        }
        todo!()
    }

    pub async fn recovery(
        factor_sources: IndexSet<HDFactorSource>,
        gateway: Arc<dyn GatewayReadonly>,
    ) -> Result<(Self, ProbablyFreeFactorInstances)> {
        let (found_entities, probably_free) = scan(
            FactorSourceDerivations::recovery(factor_sources),
            None,
            gateway,
        )
        .await?;
        todo!()
    }

    pub async fn add_factor_source(
        &mut self,
        factor_source: HDFactorSource,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        gateway: Arc<dyn GatewayReadonly>,
    ) -> Result<()> {
        let (found_entities, probably_free) = scan(
            FactorSourceDerivations::add_factor_source(factor_source),
            Some(self.scan_hook()),
            gateway,
        )
        .await?;
        todo!()
    }
}

// ===== ******* =====
// ===== HELPERS =====
// ===== ******* =====
impl Profile {
    fn entity_referencing_factor_instance(
        &self,
        factor_instance: HierarchicalDeterministicFactorInstance,
    ) -> Option<AccountOrPersona> {
        todo!()
    }
    fn scan_hook<'a>(&'a self) -> ScanHookSync<'a> {
        let call: ScanHookSync<'a> = Box::new(|fis| self._scan_hook(fis));
        call
    }
    fn _scan_hook(
        &self,
        factor_instances: IndexSet<HierarchicalDeterministicFactorInstance>,
    ) -> IndexMap<HierarchicalDeterministicFactorInstance, ScanHookDecision> {
        factor_instances
            .into_iter()
            .map(|factor_instance| {
                let decision = if let Some(entity) =
                    self.entity_referencing_factor_instance(factor_instance.clone())
                {
                    match entity.security_state() {
                        EntitySecurityState::Unsecured(_) => {
                            ScanHookDecision::UnsecurifiedEntityRecovered {
                                unsecurified_entity: entity.clone(),
                                factor_instance: factor_instance.clone(),
                            }
                        }
                        EntitySecurityState::Securified(sec) => {
                            ScanHookDecision::SecurifiedEntityReferencesFactor {
                                entity: SecurifiedEntity {
                                    entity,
                                    control: sec,
                                },
                                factor_instance: factor_instance.clone(),
                            }
                        }
                    }
                } else {
                    ScanHookDecision::ProbablyIsFree(factor_instance.clone())
                };
                (factor_instance, decision)
            })
            .collect::<IndexMap<HierarchicalDeterministicFactorInstance, ScanHookDecision>>()
    }
}

async fn scan_with_gateway(
    gateway: Arc<dyn GatewayReadonly>,
    fis: IndexSet<HierarchicalDeterministicFactorInstance>,
) -> Result<IndexMap<HierarchicalDeterministicFactorInstance, ScanHookDecisionForGateway>> {
    todo!()
}

fn scan_hook<'a>(gateway: Arc<dyn GatewayReadonly>) -> ScanHookGateway<'a> {
    Box::new(move |fis| Box::pin(scan_with_gateway(gateway.clone(), fis)))
}
