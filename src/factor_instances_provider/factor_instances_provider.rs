#![allow(unused)]
#![allow(unused_variables)]

use crate::prelude::*;

/// If both `cache` and `profile_snapshot` is `None`, then we use a derivation
/// index range starting at `0`.
///
/// Further more if cache is empty or if we are requesting derivation index ranges
/// for on a network that is not present in Profile, we start at `0`.
pub struct NextDerivationIndexAnalyzer {
    cache: Option<Arc<PreDerivedKeysCache>>,
    profile_snapshot: Option<Profile>,
}
impl NextDerivationIndexAnalyzer {
    pub fn new(
        maybe_cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
        maybe_profile_snapshot: impl Into<Option<Profile>>,
    ) -> Self {
        let cache = maybe_cache.into();
        let profile_snapshot = maybe_profile_snapshot.into();
        Self {
            cache,
            profile_snapshot,
        }
    }
}
pub struct FactorInstancesProvider {
    purpose: FactorInstancesRequestPurpose,

    /// If no cache present, a new one is created and will be filled.
    cache: Arc<PreDerivedKeysCache>,
    next_derivation_index_analyzer: NextDerivationIndexAnalyzer,

    /// GUI hook
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
}

/// ==================
/// *** CTOR ***
/// ==================

impl FactorInstancesProvider {
    fn new(
        purpose: FactorInstancesRequestPurpose,
        maybe_cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
        maybe_profile_snapshot: impl Into<Option<Profile>>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        let maybe_cache = maybe_cache.into();
        let next_derivation_index_analyzer =
            NextDerivationIndexAnalyzer::new(maybe_cache.clone(), maybe_profile_snapshot);

        let cache = maybe_cache.unwrap_or_else(|| Arc::new(PreDerivedKeysCache::default()));

        Self {
            purpose,
            cache,
            next_derivation_index_analyzer,
            derivation_interactors,
        }
    }
}

/// ==================
/// *** Purposes ***
/// ==================
impl FactorInstancesProvider {
    pub fn oars(
        factor_sources: &FactorSources,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        Self::new(
            FactorInstancesRequestPurpose::OARS {
                factor_sources: factor_sources.clone(),
            },
            None,
            None,
            derivation_interactors,
        )
    }

    pub fn mars(
        factor_source: &HDFactorSource,
        cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
        profile_snapshot: Profile,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        Self::new(
            FactorInstancesRequestPurpose::MARS {
                factor_source: factor_source.clone(),
                network_id: profile_snapshot.current_network(),
            },
            cache,
            profile_snapshot,
            derivation_interactors,
        )
    }

    pub fn pre_derive_instance_for_new_factor_source(
        factor_source: &HDFactorSource,
        cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
        profile_snapshot: Profile,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        Self::new(
            FactorInstancesRequestPurpose::PreDeriveInstancesForNewFactorSource {
                factor_source: factor_source.clone(),
            },
            cache,
            profile_snapshot,
            derivation_interactors,
        )
    }

    pub fn new_virtual_unsecurified_account(
        network_id: NetworkID,
        factor_source: &HDFactorSource,
        cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
        profile_snapshot: Profile,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        Self::new(
            FactorInstancesRequestPurpose::NewVirtualUnsecurifiedAccount {
                network_id,
                factor_source: factor_source.clone(),
            },
            cache,
            profile_snapshot,
            derivation_interactors,
        )
    }

    /// Securify unsecurified Account
    ///
    /// # Panics
    /// Panics if `UnsecurifiedEntity` is not an account
    /// or if it is not present in `profile_snapshot`.
    pub fn securify_unsecurified_account(
        unsecurified_account: UnsecurifiedEntity,
        matrix_of_factor_sources: MatrixOfFactorSources,
        cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
        profile_snapshot: Profile,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        assert!(profile_snapshot
            .contains_account(AccountAddress::try_from(unsecurified_account.clone()).unwrap()));

        Self::new(
            FactorInstancesRequestPurpose::SecurifyUnsecurifiedAccount {
                unsecurified_account,
                matrix_of_factor_sources,
            },
            cache,
            profile_snapshot,
            derivation_interactors,
        )
    }
}

/// ==================
/// *** Private API ***
/// ==================
impl FactorInstancesProvider {
    async fn load_or_derive_instances(
        &self,
        mut intermediary_analysis: &IntermediaryDerivationAndAnalysis,
    ) -> Result<()> {
        let factor_sources = self.purpose.factor_sources();
        let abstract_requests = self.purpose.requests();
        let requests_without_indices = abstract_requests.for_each_factor_source(factor_sources);
        /*
               let cached = self.cache.load(requests_without_indices).await?;

               let to_derive = IndexMap::new();

               if !cached.is_empty() {
                   let remaining = derivation_requests - cached;

                   if remaining.is_empty() {
                       /// Could satisfy derivation request from cache
                       return Ok(());
                   } else {
                       to_derive = remaining
                   }
               } else {
                   // no cache... need to determine indices to derive from Profile
                   to_derive = self
                       .profile_analyzer
                       .next_derivation_paths_fulfilling(&requests_without_indices);
               }

               /// need to derive more
               let keys_collector = KeysCollector::new(
                   self.factor_sources(),
                   remaining,
                   self.derivation_interactors,
               )?;
        */
        todo!()
    }
}

pub struct FactorInstancesRequestOutcome;

/// ==================
/// *** Public API ***
/// ==================
impl FactorInstancesProvider {
    /// The main loop of the derivation process, newly created or recovered entities,
    /// and a list of free FactorInstances - which is used to fill the cache.
    ///
    /// Gets FactorInstances either from cache or derives more, or a mix of both,
    /// until we are "done", which is either determined by End user in a callback
    /// or by the operation kind.
    pub async fn get_factor_instances(self) -> Result<FactorInstancesRequestOutcome> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        borrow::{Borrow, BorrowMut},
        sync::RwLockReadGuard,
    };

    use super::*;

    struct SargonOS {
        cache: Arc<RwLock<PreDerivedKeysCache>>,
        gateway: RwLock<TestGateway>,
        profile: RwLock<Profile>,
        interactors: Arc<dyn KeysDerivationInteractors>,
    }

    impl SargonOS {
        pub fn profile_snapshot(&self) -> Profile {
            self.profile.try_read().unwrap().clone()
        }
        pub fn new() -> Self {
            let interactors: Arc<dyn KeysDerivationInteractors> =
                Arc::new(TestDerivationInteractors::default());
            Self {
                cache: Arc::new(RwLock::new(PreDerivedKeysCache::default())),
                gateway: RwLock::new(TestGateway::default()),
                profile: RwLock::new(Profile::default()),
                interactors,
            }
        }

        async fn add_factor_source(&self, factor_source: HDFactorSource) -> Result<()> {
            let interactors: Arc<dyn KeysDerivationInteractors> =
                Arc::new(TestDerivationInteractors::default());

            let cache: Arc<PreDerivedKeysCache> = Arc::new(self.cache.try_write().unwrap().clone());

            let factor_instances_provider =
                FactorInstancesProvider::pre_derive_instance_for_new_factor_source(
                    &factor_source,
                    cache,
                    self.profile_snapshot(),
                    interactors,
                );

            factor_instances_provider.get_factor_instances().await?;

            Ok(())
        }
    }

    #[actix_rt::test]
    async fn test() {
        let os = SargonOS::new();
        // assert_eq!(os.profile_snapshot().factor_sources.len(), 0);
        // os.add_factor_source(HDFactorSource::sample())
        //     .await
        //     .unwrap();
        // assert_eq!(os.profile_snapshot().factor_sources.len(), 1);
    }
}
