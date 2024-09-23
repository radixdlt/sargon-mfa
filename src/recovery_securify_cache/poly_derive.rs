#![allow(unused)]
#![allow(unused_variables)]

use crate::prelude::*;

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
    operation_kind: PolyDeriveOperationKind,

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

/// ==================
/// *** CTOR ***
/// ==================

impl PolyDerivation {
    fn new(
        operation_kind: PolyDeriveOperationKind,
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
            operation_kind,
            cache: maybe_cache.unwrap_or_else(|| Arc::new(PreDerivedKeysCache)),
            onchain_analyser: maybe_onchain_analyser.unwrap_or_else(OnChainAnalyzer::dummy),
            profile_analyser: maybe_profile_analyser.unwrap_or_else(ProfileAnalyzer::dummy),
            derivation_interactors,
            is_derivation_done_query,
        }
    }
}

/// ==================
/// *** Operations ***
/// ==================
impl PolyDerivation {
    pub fn oars(
        factor_sources: &FactorSources,
        gateway: Arc<dyn Gateway>,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
        is_derivation_done_query: Arc<dyn IsDerivationDoneQuery>,
    ) -> Self {
        Self::new(
            PolyDeriveOperationKind::OARS {
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
            PolyDeriveOperationKind::MARS {
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
            PolyDeriveOperationKind::PreDeriveInstancesForNewFactorSource {
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
            PolyDeriveOperationKind::NewVirtualUnsecurifiedAccount {
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
            PolyDeriveOperationKind::SecurifyUnsecurifiedAccount {
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

/// ==================
/// *** Private API ***
/// ==================
impl PolyDerivation {
    /// Get next FactorInstances, either from cache or derive more, or a mix.
    ///
    /// High level description:
    /// Load FactorInstances from cache if possible, if they can satisfy
    /// the need and if cache would not be empty, used cached instance and done.
    ///
    /// If cached satisfy the need but would become empty or if cached could
    /// only partially satisfy the need, derive many more and fill cache.
    ///
    /// If cache is empty, derive many and fill cache.
    ///
    /// Satisfy "the need" is a bit tricky, we should not need to know the
    /// derivation entity indices, that is the job of the Cache (if present) and
    /// otherwise the ProfileAnalyzer. In case of no cache and no Prifile, we use
    /// a range of `(0, SIZE)`.
    ///
    /// In detail:
    /// Form FactorSource and IndexRange agnostic derivation requests from
    /// the operation kind.
    ///
    /// Form IndexRange agnostic derivation requests from the FactorSource agnostic
    /// ones.
    ///
    /// Try use "next indices" from cache, if not possible, then use ProfileAnalyzer,
    /// if not present, then use a range of `(0, SIZE)`, where `SIZE` is typically `30`
    /// (might depend on FactorSourceKind... which would complicate this
    /// implementation quite a bit).
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

    /// Ask GUI callback/hook if derivation is done, which is async because we should
    /// it to end user for some operations, e.g. showing the derived Accounts so far
    /// during (Onboarding/Manual) Account Recover Scan.
    async fn is_done(&self, derived_accounts: &DerivedFactorInstances) -> Result<bool> {
        self.is_derivation_done_query
            .is_done(derived_accounts)
            .await
    }

    /// Get the Agnostic derivation requests for the operation kind.
    fn requests(&self) -> AnyFactorDerivationRequests {
        self.operation_kind.requests()
    }

    /// Get the FactorSources for the operation kind.
    fn factor_sources(&self) -> FactorSources {
        self.operation_kind.factor_sources()
    }

    /// The instances derived or loaded from cache, or a mix of both, which
    /// are relevant for the operation kind.
    fn derived_instances(&self) -> Result<DerivedFactorInstances> {
        // Implementation wise we use a simplistic approach, we use
        // the cache as the storage of any newly derived instances,
        // or if the existing instances in cache could fulfill the
        // requests of the operation, then only those are used.
        todo!("get instances from cache")
    }
}

/// ==================
/// *** Public API ***
/// ==================
impl PolyDerivation {
    /// The main loop of the derivation process, newly created or recovered entities,
    /// and a list of free FactorInstances - which is used to fill the cache.
    ///
    /// Gets FactorInstances either from cache or derives more, or a mix of both,
    /// until we are "done", which is either determined by End user in a callback
    /// or by the operation kind.
    pub async fn poly_derive(self) -> Result<FinalDerivationsAndAnalysis> {
        loop {
            let derived = self.derived_instances()?;
            let is_done = self.is_done(&derived).await?;
            if is_done {
                break;
            }
            self.load_or_derive_instances().await?;
        }

        let derived_instances = self.derived_instances()?;
        let cache = self.cache;

        let analysis = FinalDerivationsAndAnalysis {
            derived_instances,
            cache,
        };

        Ok(analysis)
    }
}

/// OARS: Onboarding account recover scan
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

/// MARS: Manual account recovery scan
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

/// PreDerive for new FactorSource
/// Host app would first create the FactorSource, unsaved,
/// and pass it here. For Ledger, Host App should first establish
/// a connection, read out the FactorSourceIDFromHash, possibly
/// then derive and after that name the FactorSource (updating the name),
/// since this method saves it into Profile.
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

/// Create New Virtual Unsecurified Account
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
