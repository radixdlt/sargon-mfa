use crate::prelude::*;

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
        cache: Option<Arc<RwLock<PreDerivedKeysCache>>>,
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
        cache: Option<Arc<RwLock<PreDerivedKeysCache>>>,
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
        cache: Option<Arc<RwLock<PreDerivedKeysCache>>>,
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

    /// Update securified accounts or securify unsecurified accounts
    pub fn update_or_set_security_shield_for_accounts(
        accounts: Accounts,
        matrix_of_factor_sources: MatrixOfFactorSources,
        cache: Option<Arc<RwLock<PreDerivedKeysCache>>>,
        profile_snapshot: Profile,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Self {
        assert!(profile_snapshot.contains_accounts(accounts.clone()));

        Self::new(
            FactorInstancesRequestPurpose::UpdateOrSetSecurityShieldForAccounts {
                accounts,
                matrix_of_factor_sources,
            },
            cache,
            profile_snapshot,
            derivation_interactors,
        )
    }
}
