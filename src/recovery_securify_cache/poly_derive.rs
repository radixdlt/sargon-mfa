#![allow(unused)]
#![allow(unused_variables)]

use std::net;

use crate::prelude::*;

pub enum PolyDeriveRequestKind {
    /// Onboarding Account Recovery Scan
    /// Assumes `Mainnet`
    OARS { factor_sources: FactorSources },

    /// Manual Account Recovery Scan
    /// Done using a single FactorSource
    MARS {
        factor_source: HDFactorSource,
        network_id: NetworkID,
    },

    /// PreDerive FactorInstances for new FactorSource
    PreDeriveInstancesForNewFactorSource { factor_source: HDFactorSource },

    /// New Virtual Unsecurified Account
    NewVirtualUnsecurifiedAccount {
        network_id: NetworkID,
        factor_source: HDFactorSource,
    },

    /// Securify unsecurified Account
    SecurifyUnsecurifiedAccount {
        unsecurified_account: UnsecurifiedEntity,
        matrix_of_factor_sources: MatrixOfFactorSources,
    },

    /// Securify unsecurified Account
    UpdateSecurifiedAccount {
        securified_account: SecurifiedEntity,
        matrix_of_factor_sources: MatrixOfFactorSources,
    },
}
impl PolyDeriveRequestKind {
    pub fn factor_sources(&self) -> FactorSources {
        match self {
            Self::OARS { factor_sources } => factor_sources.clone(),
            Self::MARS { factor_source, .. } => FactorSources::just(factor_source.clone()),
            Self::PreDeriveInstancesForNewFactorSource { factor_source } => {
                FactorSources::just(factor_source.clone())
            }
            Self::NewVirtualUnsecurifiedAccount { factor_source, .. } => {
                FactorSources::just(factor_source.clone())
            }
            Self::SecurifyUnsecurifiedAccount {
                matrix_of_factor_sources,
                ..
            } => matrix_of_factor_sources
                .all_factors()
                .into_iter()
                .collect::<FactorSources>(),
            Self::UpdateSecurifiedAccount {
                matrix_of_factor_sources,
                ..
            } => matrix_of_factor_sources
                .all_factors()
                .into_iter()
                .collect::<FactorSources>(),
        }
    }
}

/// Derivation of many keys differs between the following operations,
///
/// I will introduce some new concepts:
/// [VECI]: Virtual Entity Creating (Factor)Instance - previously (in meetings/
/// Slack) called "genesis factor instances", which is the FactorInstance which
/// created a virtual entity and formed its address. This is set in Profile on
/// said entity when it is securified to help with "is instance free" queries
/// during Profile analysis. It must be Optional since it might be unknown for
/// **recovered** securified entities - which can happen if the FactorSource of
/// the VECI was not provided by the user - or if not broad enough an index
/// space was scanned.
///
/// All operations ends with adding "ProbablyFree" FactorInstances to PreDerivedKeysCache.
///
/// "ProbablyFree" refers to keys in `KeySpace::Securified`, a Factor which
/// might have been used in the past for an AccessController, but might not
/// be active anymore - currently [2024-09-20] it does not look like Gateway
/// will index **past** public key hashes.
///
/// If cache is empty/non-existent/inaccessible, we derive many keys for many
/// derivation index ranges to fill the cache and when possible/applicable we
/// match against entities in Profile to re-discovered VECIs and unsecurified
/// entities, also when possible/applicable we analyze if FactorInstances are
/// taken or free by using Gateway.
///
/// OPERATIONS:
///
/// * [FSA] FactorSource AdditionAddition
/// Does NOT add any new entities, for that, use `MARS`.`
///     CHARACTERISTICS:
///     - Derivation Indices Range Start: `0`
///     - Derivation Indices Range Size: Many, different for each factor source kind and key kind
///     - PreDerivedKeysCache Available: NO, not for this FactorSource
///     - Profile Available: YES
///     - FactorSource Addition: `Single`
///     - Entities Addition: NO
///     - `VECI` Addition: YES - if new found
///     - Gateway Required: NO, not used.
///
///
/// * [OARS] Onboarding Account Recovery Scan - like `FSA` but for many FactorSources and
///     with Gateway Required.
///     CHARACTERISTICS:
///     - Derivation Indices Range Start: `0`
///     - Derivation Indices Range Size: Many, different for each factor source kind and key kind
///     - PreDerivedKeysCache Available: NO
///     - Profile Available: NO
///     - FactorSource Addition: `Many`
///     - Entities Addition: YES - securified and unsecurified
///     - `VECI` Addition: YES
///     - Gateway Required: YES
///
///
/// * [MARS] Manual Account Recovery Scan
///     CHARACTERISTICS:
///     - Derivation Indices Range Start: `0`
///     - Derivation Indices Range Size: Many, different for each factor source kind and key kind
///     - PreDerivedKeysCache Available: NO
///     - Profile Available: NO
///     - FactorSource Addition: `Many`
///     - Entities Addition: YES - securified and unsecurified
///     - `VECI` Addition: YES
///     - Gateway Required: YES
///
///
/// * [NUVEC] New Unsecurified Virtual Entity Creation
///     CHARACTERISTICS:
///     - Derivation Indices Range Start: Next Free According to PreDerivedKeysCache if available, else start at Zero to fill cache and filter out take (by matching against entities in Profile)
///     - Derivation Indices Range Size: Single (OR Many if PreDerivedKeysCache needs to be filled)
///     - PreDerivedKeysCache Available: YES - if not deleted or inaccessible
///     - Profile Available: YES
///     - FactorSource Addition: NO
///     - Entities Addition: YES - unsecurified
///     - `VECI` Addition: NO - cannot happen, PreDerivedKeysCache should never contain keys which have been used to create Account/Identity addresses.
///     - Gateway Required: NO (but beneficial to use it if host is online to
///         analyze if FactorInstance are free.)
///
///
/// * [MOFIDs] `MatrixOfFactorSources` -> `Vec<MatrixOfFactorInstances>` Derivation (Securifying Entities)
///     CHARACTERISTICS:
///     - Derivation Indices Range Start: Next Free According to PreDerivedKeysCache if available, else start at Zero to fill cache and filter out take (by matching against entities in Profile)
///     - Derivation Indices Range Size: Single PER FactorSource PER Entity being securified (OR Many if PreDerivedKeysCache needs to be filled)
///     - PreDerivedKeysCache Available: YES - if not deleted or inaccessible
///     - Profile Available: YES
///     - FactorSource Addition: NO
///     - Entities Addition: NO
///     - `VECI` Addition: YES - if cache was empty, we might have re-discovered a VECI.
///     - Gateway Required: NO (but beneficial to use it if host is online to
///         analyze if FactorInstance are free.)
///
///
pub struct PolyDerivation {
    request_kind: PolyDeriveRequestKind,

    /// If no cache present, a new one is created and will be filled.
    cache: Arc<PreDerivedKeysCache>,

    /// If not present (no Gateway) or if offline, a "dummy" one is used
    /// which says everything is free.
    onchain_analyser: OnChainAnalyzer,

    /// If not present (no Profile) a dummy one is used which says everything is free.
    profile_analyser: ProfileAnalyzer,

    /// GUI hooks
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    is_derivation_done_query: Arc<dyn IsDerivationDoneQuery>,
}

impl PolyDerivation {
    fn new(
        request_kind: PolyDeriveRequestKind,
        maybe_cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
        maybe_onchain_analyser: impl Into<Option<OnChainAnalyzer>>,
        maybe_profile_analyser: impl Into<Option<ProfileAnalyzer>>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        is_derivation_done_query: Arc<dyn IsDerivationDoneQuery>,
    ) -> Self {
        let maybe_cache = maybe_cache.into();
        let maybe_onchain_analyser = maybe_onchain_analyser.into();
        let maybe_profile_analyser = maybe_profile_analyser.into();

        assert!(
            !(maybe_cache.is_none()
                && maybe_onchain_analyser.is_none()
                && maybe_profile_analyser.is_none())
        );
        Self {
            request_kind,
            cache: maybe_cache.unwrap_or_else(|| Arc::new(PreDerivedKeysCache)),
            onchain_analyser: maybe_onchain_analyser.unwrap_or_else(OnChainAnalyzer::dummy),
            profile_analyser: maybe_profile_analyser.unwrap_or_else(ProfileAnalyzer::dummy),
            derivation_interactors,
            is_derivation_done_query,
        }
    }

    pub fn oars(
        factor_sources: &FactorSources,
        gateway: Arc<dyn Gateway>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        is_derivation_done_query: Arc<dyn IsDerivationDoneQuery>,
    ) -> Self {
        Self::new(
            PolyDeriveRequestKind::OARS {
                factor_sources: factor_sources.clone(),
            },
            None,
            OnChainAnalyzer::with_gateway(gateway),
            None,
            derivation_interactors,
            is_derivation_done_query,
        )
    }

    pub fn mars(
        factor_source: &HDFactorSource,
        gateway: Arc<dyn Gateway>,
        cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
        profile: Arc<Profile>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        is_derivation_done_query: Arc<dyn IsDerivationDoneQuery>,
    ) -> Self {
        Self::new(
            PolyDeriveRequestKind::MARS {
                factor_source: factor_source.clone(),
                network_id: profile.current_network(),
            },
            cache,
            OnChainAnalyzer::with_gateway(gateway),
            ProfileAnalyzer::with_profile(profile),
            derivation_interactors,
            is_derivation_done_query,
        )
    }

    pub fn pre_derive_instance_for_new_factor_source(
        factor_source: &HDFactorSource,
        gateway: impl Into<Option<Arc<dyn Gateway>>>,
        cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
        profile: Arc<Profile>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        Self::new(
            PolyDeriveRequestKind::PreDeriveInstancesForNewFactorSource {
                factor_source: factor_source.clone(),
            },
            cache,
            OnChainAnalyzer::new(gateway),
            ProfileAnalyzer::with_profile(profile),
            derivation_interactors,
            Arc::new(YesDone),
        )
    }

    pub fn new_virtual_unsecurified_account(
        network_id: NetworkID,
        factor_source: &HDFactorSource,
        gateway: impl Into<Option<Arc<dyn Gateway>>>,
        cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
        profile: Arc<Profile>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        Self::new(
            PolyDeriveRequestKind::NewVirtualUnsecurifiedAccount {
                network_id,
                factor_source: factor_source.clone(),
            },
            cache,
            OnChainAnalyzer::new(gateway),
            ProfileAnalyzer::with_profile(profile),
            derivation_interactors,
            Arc::new(YesDone),
        )
    }

    /// Securify unsecurified Account
    pub fn securify_unsecurified_account(
        account_address: AccountAddress,
        matrix_of_factor_sources: MatrixOfFactorSources,
        gateway: impl Into<Option<Arc<dyn Gateway>>>,
        cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
        profile: Arc<Profile>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        let unsecurified_account = profile
            .get_account(&account_address)
            .unwrap()
            .as_unsecurified()
            .unwrap()
            .clone();

        Self::new(
            PolyDeriveRequestKind::SecurifyUnsecurifiedAccount {
                unsecurified_account,
                matrix_of_factor_sources,
            },
            cache,
            OnChainAnalyzer::new(gateway),
            ProfileAnalyzer::with_profile(profile),
            derivation_interactors,
            Arc::new(YesDone),
        )
    }
}

#[async_trait::async_trait]
pub trait IsDerivationDoneQuery {
    async fn is_done(&self, derived_accounts: &DerivedFactorInstances) -> Result<bool>;
}

pub struct YesDone;
#[async_trait::async_trait]
impl IsDerivationDoneQuery for YesDone {
    async fn is_done(&self, derived_accounts: &DerivedFactorInstances) -> Result<bool> {
        Ok(true)
    }
}

impl PolyDerivation {
    async fn is_done(&self, derived_accounts: &DerivedFactorInstances) -> Result<bool> {
        self.is_derivation_done_query
            .is_done(derived_accounts)
            .await
    }

    fn requests(&self) -> AnyFactorDerivationRequests {
        todo!()
    }

    fn factor_sources(&self) -> FactorSources {
        self.request_kind.factor_sources()
    }

    async fn load_or_derive_instances(&self) -> Result<()> {
        let factor_sources = self.factor_sources();
        let abstract_requests = self.requests();
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
                       .profile_analyser
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

    fn derived_instances(&self) -> DerivedFactorInstances {
        // self.cache.recovered_accounts()
        todo!()
    }

    pub async fn poly_derive(self) -> Result<FinalDerivationsAndAnalysis> {
        loop {
            let is_done = self.is_done(&self.derived_instances()).await?;
            if is_done {
                break;
            }
            self.load_or_derive_instances().await?;
        }

        let derived_instances = self.derived_instances();
        let cache = self.cache;

        let analysis = FinalDerivationsAndAnalysis {
            derived_instances,
            cache,
        };

        Ok(analysis)
    }
}

/// onboarding account recover scan
pub async fn oars(
    factor_sources: FactorSources,
    interactors: Arc<dyn KeysDerivationInteractors>,
    gateway: Arc<dyn Gateway>,
    is_derivation_done_query: Arc<dyn IsDerivationDoneQuery>,
) -> Result<(Profile, Arc<PreDerivedKeysCache>)> {
    let network_id = NetworkID::Mainnet;

    let derivation = PolyDerivation::oars(
        &factor_sources,
        gateway,
        interactors,
        is_derivation_done_query,
    );

    let analysis = derivation.poly_derive().await?;
    let cache = analysis.cache;

    let recovered_unsecurified_accounts =
        analysis.derived_instances.accounts_unsecurified(network_id);

    // TODO handle securified!
    let profile = Profile::new(factor_sources, &recovered_unsecurified_accounts, []);

    Ok((profile, cache))
}

pub async fn mars(
    factor_source: HDFactorSource,
    interactors: Arc<dyn KeysDerivationInteractors>,
    gateway: Arc<dyn Gateway>,
    profile: &mut Profile,
    cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
    is_derivation_done_query: Arc<dyn IsDerivationDoneQuery>,
) -> Result<Arc<PreDerivedKeysCache>> {
    let network_id = profile.current_network();
    let derivation = PolyDerivation::mars(
        &factor_source,
        gateway,
        cache,
        Arc::new(profile.clone()),
        interactors,
        is_derivation_done_query,
    );

    let analysis = derivation.poly_derive().await?;
    let cache = analysis.cache;
    let accounts = analysis.derived_instances.accounts_unsecurified(network_id);

    profile.insert_accounts(accounts)?;

    Ok(cache)
}

pub async fn pre_derive_instance_for_new_factor_source(
    factor_source: &HDFactorSource, // not yet added to Profile.
    gateway: impl Into<Option<Arc<dyn Gateway>>>,
    cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
    profile: &mut Profile,
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
) -> Result<Arc<PreDerivedKeysCache>> {
    let network_id = profile.current_network();
    let derivation = PolyDerivation::pre_derive_instance_for_new_factor_source(
        factor_source,
        gateway,
        cache,
        Arc::new(profile.clone()),
        derivation_interactors,
    );

    let analysis = derivation.poly_derive().await?;
    let cache = analysis.cache;
    profile.add_factor_source(factor_source.clone())?;

    Ok(cache)
}

pub async fn new_virtual_unsecurified_account(
    name: impl AsRef<str>,
    network_id: NetworkID,
    factor_source: &HDFactorSource,
    gateway: impl Into<Option<Arc<dyn Gateway>>>,
    cache: impl Into<Option<Arc<PreDerivedKeysCache>>>,
    profile: &mut Profile,
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
) -> Result<Account> {
    let network_id = profile.current_network();
    let derivation = PolyDerivation::new_virtual_unsecurified_account(
        network_id,
        factor_source,
        gateway,
        cache,
        Arc::new(profile.clone()),
        derivation_interactors,
    );

    let analysis = derivation.poly_derive().await?;

    let mut account = analysis
        .derived_instances
        .accounts_unsecurified(network_id)
        .first()
        .ok_or(CommonError::UnknownAccount)
        .cloned()?;

    account.set_name(name);

    profile.insert_accounts(IndexSet::from_iter([account.clone()]))?;

    Ok(account)
}
