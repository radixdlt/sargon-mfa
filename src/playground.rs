#![allow(unused)]

use crate::prelude::*;

/// "Probably" since we might not have all the information to be sure, since
/// Gateway might not keep track of past FactorInstances, some of the FactorInstances
/// in KeySpace::Securified might in fact have been used in the past for some entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbablyFreeFactorInstances(IndexSet<HierarchicalDeterministicFactorInstance>);

enum ScanHookDecision {
    /// "Probably" since we might not have all the information to be sure, since
    /// Gateway might not keep track of past FactorInstances, some of the FactorInstances
    /// in KeySpace::Securified might in fact have been used in the past for some entity.
    ProbablyIsFree(HierarchicalDeterministicFactorInstance),
    UnsecurifiedEntityRecovered(AccountOrPersona),
    SecurifiedEntityReferencesFactor {
        entity_address: AddressOfAccountOrPersona,
        factor_instance: HierarchicalDeterministicFactorInstance,
    },
}

type OnFactorInstance = Box<
    dyn FnOnce(
        HierarchicalDeterministicFactorInstance,
    ) -> Pin<Box<dyn Future<Output = ScanHookDecision>>>,
>;
struct ScanHook {
    on_factor_instance: OnFactorInstance,
}

async fn scan(
    factor_sources: IndexSet<HDFactorSource>,
    profile_scan_hook: impl Into<Option<ScanHook>>,
    gateway_scan_hook: impl Into<Option<ScanHook>>,
) -> Result<(IndexSet<AccountOrPersona>, ProbablyFreeFactorInstances)> {
    todo!()
}
impl dyn GatewayReadonly {
    fn scan_hook(&self) -> ScanHook {
        todo!()
    }
}
impl Profile {
    fn scan_hook(&self) -> ScanHook {
        todo!()
    }

    async fn add_factor_source(
        &mut self,
        factor_source: HDFactorSource,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        gateway_scan_hook: ScanHook,
    ) -> Result<()> {
        let (found_entities, probably_free) = scan(
            IndexSet::just(factor_source),
            self.scan_hook(),
            gateway_scan_hook,
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
        self.add_factor_source(factor_source, derivation_interactors, gateway.scan_hook())
            .await
    }
}

async fn recovery(
    factor_sources: IndexSet<HDFactorSource>,
    gateway_scan_hook: ScanHook,
) -> Result<(Profile, ProbablyFreeFactorInstances)> {
    let (found_entities, probably_free) = scan(factor_sources, None, gateway_scan_hook).await?;
    todo!()
}

async fn recovery_with_gateway(
    factor_sources: IndexSet<HDFactorSource>,
    gateway: Arc<dyn GatewayReadonly>,
) -> Result<(Profile, ProbablyFreeFactorInstances)> {
    recovery(factor_sources, gateway.scan_hook()).await
}
