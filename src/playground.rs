#![allow(unused)]

use std::ops::Deref;

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

async fn scan<'p, 'g>(
    factor_sources: IndexSet<HDFactorSource>,
    profile_scan_hook: Option<ScanHookSync<'p>>,
    gateway_scan_hook: Option<ScanHookGateway<'g>>,
) -> Result<(IndexSet<AccountOrPersona>, ProbablyFreeFactorInstances)> {
    todo!()
}

impl Profile {
    async fn add_factor_source<'g>(
        &mut self,
        factor_source: HDFactorSource,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        gateway_scan_hook: ScanHookGateway<'g>,
    ) -> Result<()> {
        let (found_entities, probably_free) = scan(
            IndexSet::just(factor_source),
            Some(self.scan_hook()),
            Some(gateway_scan_hook),
        )
        .await?;
        todo!()
    }

    async fn add_factor_source_with_gateway(
        &mut self,
        factor_source: HDFactorSource,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        gateway: Arc<dyn GatewayReadonly>,
    ) -> Result<()> {
        self.add_factor_source(factor_source, derivation_interactors, scan_hook(gateway))
            .await
    }

    async fn recovery(
        factor_sources: IndexSet<HDFactorSource>,
        gateway_scan_hook: ScanHookGateway<'_>,
    ) -> Result<(Self, ProbablyFreeFactorInstances)> {
        let (found_entities, probably_free) =
            scan(factor_sources, None, Some(gateway_scan_hook)).await?;
        todo!()
    }

    async fn recovery_with_gateway(
        factor_sources: IndexSet<HDFactorSource>,
        gateway: Arc<dyn GatewayReadonly>,
    ) -> Result<(Self, ProbablyFreeFactorInstances)> {
        Self::recovery(factor_sources, scan_hook(gateway)).await
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
