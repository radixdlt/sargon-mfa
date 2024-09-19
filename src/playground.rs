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

/// "VECI"
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VirtualEntityCreatingFactorInstance {
    validated: bool,
    /// The address of the entity that `factor_instance` has created
    pub address: AddressOfAccountOrPersona,
    /// "VECI"
    pub factor_instance: HierarchicalDeterministicFactorInstance,
}
impl VirtualEntityCreatingFactorInstance {
    pub fn new(
        address: AddressOfAccountOrPersona,
        factor_instance: HierarchicalDeterministicFactorInstance,
    ) -> Self {
        assert!(
            AddressOfAccountOrPersona::new(
                factor_instance.clone(),
                address.network_id(),
                address.entity_kind()
            ) == address,
            "Discrepancy! Non matching address, this is a programmer error"
        );
        Self {
            validated: true,
            address,
            factor_instance,
        }
    }
}

/// A collection of FactorInstances which we matches against entities already
/// existing in Profile, either the FactorInstances was already in Profile, in which
/// case it will be put in `skipped_instances_since_already_known`, or it was
/// not in Profile but we managed to match it against the address of an securified
/// entity, in which case it will be put in `rediscovered_virtually_creating_instances`.
///
/// Any instance which was not matched against Profile will not be in this struct,
/// i.e. if we rediscovered an Entity that was not in Profile, it will be part of
/// `FactorInstancesAnalysis` (just like this struct), but in some other field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactorInstancesAnalysisMatchedAgainstProfile {
    /// Set of FactorInstances which we managed to match against securified
    /// entities as their Virtual Entity Creating Instance.
    pub rediscovered_virtually_creating_instances: IndexSet<VirtualEntityCreatingFactorInstance>,

    /// FactorInstances which we skipped since they were already known to Profile.
    pub skipped_instances_since_already_known: IndexSet<HierarchicalDeterministicFactorInstance>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactorInstancesAnalysisMatchedAgainstGateway {
    /// The On-Chain information about entities referencing a certain public
    /// key hash merged together with the FactorInstances from which the
    /// public key hash was created.
    ///
    /// N.B. We REALLY would like to be able to represent this field as:
    /// `IndexMap<HierarchicalDeterministicFactorInstance, OnChainEntityInformation>`
    /// i.e. a **single** `OnChainEntityInformation` per instance, but it is
    /// possible that the same key was used on two different entities - something
    /// the wallet will try hard to avoid - but still possible, thus we must have
    /// a set here
    pub info_by_factor_instances:
        IndexMap<HierarchicalDeterministicFactorInstance, IndexSet<OnChainEntityInformation>>,
}

/// Analysis of the FactorInstances that were derived recently for a list of
/// FactorSources, and how they match against the Profile and Gateway.
///
/// This analysis was done either for "Recovery" or "Add-FactorSource" and also
/// by both "NewVirtualEntity" and "SecurifyEntity" if no cache exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactorInstancesAnalysis {
    /// Collection of FactorInstances which did not match against Profile or Gateway,
    /// and are thus probably free. "Probably" since we Gateway might not keep track
    /// of **past** public key hashes, in which case the FactorInstances might
    /// in fact have been used by some entity in the past, but since been removed.
    pub probably_free_instances: ProbablyFreeFactorInstances,

    /// `None` if the analysis was not done against a Profile, i.e. during recovery,
    /// `Some` if the analysis was done against a Profile and the result of
    /// any factor instances matched against Profile.
    pub matched_against_profile: Option<FactorInstancesAnalysisMatchedAgainstProfile>,

    /// `None` if we have not internet connection.
    /// `Some` if we had an internet connection and successfully managed to scan
    /// Gateway for the FactorInstances.
    pub matched_against_gateway: Option<FactorInstancesAnalysisMatchedAgainstGateway>,
}

/// Used by "Recovery" and "Add-FactorSource" and also by both "NewVirtualEntity"
/// and "SecurifyEntity" if no cache exists.
///
/// Pre-Derives many FactorInstances for the new FactorSource, for many different
/// "DerivationRequests":
///     * NetworkID
///     * EntityKind
///     * KeyKind
///     * KeySpace
/// And for each of those DerivationRequest many indices, we start at "next index",
/// which is non-trivially calculated.
///
/// For each FactorInstances we check if it was already known to either:
/// * Profile  
///     - might be the `VECI` (Virtual Entity Creating Instance) of a securified
///         entity => set the `VECI`
/// * Gateway
///     * might have been used in a securified entity in the past => avoid it
///     * might control a non-recovered unsecurified entity => recover it
///
/// For all unknown FactorInstances => put them in the cache, keyed under the
/// FactorSourceID and under the DerivationRequest.
async fn derive_and_analyze_status_of_factor_instances(
    factor_source_derivations: FactorSourceDerivations,
    profile_scan_hook: Option<ScanHookSync<'_>>,
    gateway: Arc<dyn GatewayReadonly>,
) -> Result<FactorInstancesAnalysis> {
    todo!()
}

impl Profile {
    fn add_all_entities(&mut self, entities: IndexSet<AccountOrPersona>) {
        todo!()
    }
    async fn new_entity<E: IsEntity + std::fmt::Debug + std::hash::Hash + Eq>(
        &mut self,
        network_id: NetworkID,
        name: impl AsRef<str>,
        factor_source_id: FactorSourceIDFromHash,
        factor_instance_provider: &FactorInstanceProvider,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<E> {
        assert!(self
            .factor_sources
            .iter()
            .map(|f| f.factor_source_id())
            .contains(&factor_source_id));

        let genesis_factor = factor_instance_provider
            .provide_genesis_factor_for(
                factor_source_id,
                E::kind(),
                network_id,
                self,
                derivation_interactors,
            )
            .await?;

        let address = E::Address::by_hashing(network_id, genesis_factor.clone());

        let entity = E::new(
            name,
            address,
            EntitySecurityState::Unsecured(genesis_factor),
        );

        let erased = Into::<AccountOrPersona>::into(entity.clone());

        match erased {
            AccountOrPersona::AccountEntity(account) => {
                self.accounts.insert(account.entity_address(), account);
            }
            AccountOrPersona::PersonaEntity(persona) => {
                self.personas.insert(persona.entity_address(), persona);
            }
        };

        Ok(entity)
    }

    pub async fn new_account(
        &mut self,
        network_id: NetworkID,
        name: impl AsRef<str>,
        factor_source_id: FactorSourceIDFromHash,
        factor_instance_provider: &FactorInstanceProvider,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<Account> {
        self.new_entity(
            network_id,
            name,
            factor_source_id,
            factor_instance_provider,
            derivation_interactors,
        )
        .await
    }

    /// Creates a new Profile with the list of factor sources, and all
    /// entities that were recovered from the factor sources.
    ///
    /// The newly created Profile is returned together with a list of
    /// "probably free" factor instances which should be put in a cache.
    pub async fn recovery(
        factor_sources: IndexSet<HDFactorSource>,
        gateway: Arc<dyn GatewayReadonly>,
    ) -> Result<(Self, ProbablyFreeFactorInstances)> {
        let analysis = derive_and_analyze_status_of_factor_instances(
            FactorSourceDerivations::recovery(factor_sources),
            None,
            gateway,
        )
        .await?;
        todo!()
    }

    /// Adds a factor source by deriving many factor instances and analyzing
    /// them against the Profile and Gateway.
    ///
    /// Any newly recovered entities of re-discovered VECIs will be put in profile,
    /// all "probably free" factor instances should be put in a cache.
    pub async fn add_factor_source(
        &mut self,
        factor_source: HDFactorSource,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        gateway: Arc<dyn GatewayReadonly>,
    ) -> Result<ProbablyFreeFactorInstances> {
        let analysis = derive_and_analyze_status_of_factor_instances(
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
