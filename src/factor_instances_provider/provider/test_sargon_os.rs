#![cfg(test)]

use crate::prelude::*;

pub(super) struct SargonOS {
    cache: Cache,
    profile: RwLock<Profile>,
}

impl SargonOS {
    pub(super) fn profile_snapshot(&self) -> Profile {
        self.profile.try_read().unwrap().clone()
    }

    pub(super) fn new() -> Self {
        Arc::new(TestDerivationInteractors::default());
        Self {
            cache: Cache::default(),
            profile: RwLock::new(Profile::default()),
        }
    }

    pub(super) async fn with_bdfs() -> (Self, HDFactorSource) {
        let mut self_ = Self::new();
        let bdfs = HDFactorSource::device();
        self_.add_factor_source(bdfs.clone()).await.unwrap();
        (self_, bdfs)
    }

    pub(super) fn cache_snapshot(&self) -> Cache {
        self.cache.clone()
    }

    pub(super) fn clear_cache(&mut self) {
        println!("💣 CLEAR CACHE");
        self.cache = Cache::default()
    }

    pub(super) async fn new_mainnet_account_with_bdfs(
        &mut self,
        name: impl AsRef<str>,
    ) -> Result<(Account, FactorInstancesProviderOutcomeForFactor)> {
        self.new_account_with_bdfs(NetworkID::Mainnet, name).await
    }

    pub(super) async fn new_account_with_bdfs(
        &mut self,
        network: NetworkID,
        name: impl AsRef<str>,
    ) -> Result<(Account, FactorInstancesProviderOutcomeForFactor)> {
        let bdfs = self.profile_snapshot().bdfs();
        self.new_account(bdfs, network, name).await
    }

    pub(super) async fn new_account(
        &mut self,
        factor_source: HDFactorSource,
        network: NetworkID,
        name: impl AsRef<str>,
    ) -> Result<(Account, FactorInstancesProviderOutcomeForFactor)> {
        let profile_snapshot = self.profile_snapshot();
        let outcome = FactorInstancesProvider::for_account_veci(
            &mut self.cache,
            Some(profile_snapshot),
            factor_source.clone(),
            network,
            Arc::new(TestDerivationInteractors::default()),
        )
        .await
        .unwrap();

        let outcome_for_factor = outcome
            .per_factor
            .get(&factor_source.factor_source_id())
            .unwrap()
            .clone();

        let instances_to_use_directly = outcome_for_factor.to_use_directly.clone();

        assert_eq!(instances_to_use_directly.len(), 1);
        let instance = instances_to_use_directly.first().unwrap();

        let address = AccountAddress::new(network, instance.public_key_hash());
        let account = Account::new(
            name,
            address,
            EntitySecurityState::Unsecured(instance),
            ThirdPartyDepositPreference::default(),
        );
        self.profile
            .try_write()
            .unwrap()
            .add_account(&account)
            .unwrap();
        Ok((account, outcome_for_factor))
    }

    pub(super) async fn securify(
        &mut self,
        accounts: Accounts,
        shield: MatrixOfFactorSources,
    ) -> Result<(SecurifiedAccounts, FactorInstancesProviderOutcome)> {
        let profile_snapshot = self.profile_snapshot();

        let outcome = FactorInstancesProvider::for_account_mfa(
            &mut self.cache,
            shield.clone(),
            profile_snapshot,
            accounts
                .clone()
                .into_iter()
                .map(|a| a.entity_address())
                .collect(),
            Arc::new(TestDerivationInteractors::default()),
        )
        .await
        .unwrap();

        let mut instance_per_factor = outcome
            .clone()
            .per_factor
            .into_iter()
            .map(|(k, outcome_per_factor)| (k, outcome_per_factor.to_use_directly))
            .collect::<IndexMap<FactorSourceIDFromHash, FactorInstances>>();

        // Now we need to map the flat set of instances into many MatrixOfFactorInstances, and assign
        // one to each account
        let updated_accounts = accounts
            .clone()
            .into_iter()
            .map(|a| {
                let matrix_of_instances =
                    MatrixOfFactorInstances::fulfilling_matrix_of_factor_sources_with_instances(
                        &mut instance_per_factor,
                        shield.clone(),
                    )
                    .unwrap();
                let access_controller = match a.security_state() {
                    EntitySecurityState::Unsecured(_) => {
                        AccessController::from_unsecurified_address(a.entity_address())
                    }
                    EntitySecurityState::Securified(sec) => sec.access_controller.clone(),
                };
                let veci = match a.security_state() {
                    EntitySecurityState::Unsecured(veci) => Some(veci),
                    EntitySecurityState::Securified(sec) => sec.veci.clone(),
                };
                let sec =
                    SecurifiedEntityControl::new(matrix_of_instances, access_controller, veci);
                SecurifiedAccount::new(a.name(), a.entity_address(), sec, a.third_party_deposit())
            })
            .collect::<IndexSet<SecurifiedAccount>>();

        for account in updated_accounts.iter() {
            self.profile
                .try_write()
                .unwrap()
                .update_account(&account.account())
                .unwrap();
        }
        assert!(
            instance_per_factor.values().all(|x| x.is_empty()),
            "should have used all instances, but have unused instances: {:?}",
            instance_per_factor
        );
        SecurifiedAccounts::new(accounts.network_id(), updated_accounts).map(|x| (x, outcome))
    }

    pub(super) async fn add_factor_source(&mut self, factor_source: HDFactorSource) -> Result<()> {
        let profile_snapshot = self.profile_snapshot();
        assert!(
            !profile_snapshot
                .factor_sources
                .iter()
                .any(|x| x.factor_source_id() == factor_source.factor_source_id()),
            "factor already in Profile"
        );
        let outcome = FactorInstancesProvider::for_new_factor_source(
            &mut self.cache,
            Some(profile_snapshot),
            factor_source.clone(),
            NetworkID::Mainnet,
            Arc::new(TestDerivationInteractors::default()),
        )
        .await
        .unwrap();

        let per_factor = outcome.per_factor;
        let outcome = per_factor
            .get(&factor_source.factor_source_id())
            .unwrap()
            .clone();
        assert_eq!(outcome.factor_source_id, factor_source.factor_source_id());

        assert_eq!(outcome.found_in_cache.len(), 0);

        assert_eq!(
            outcome.to_cache.len(),
            NetworkIndexAgnosticPath::all_presets().len() * CACHE_FILLING_QUANTITY
        );

        assert_eq!(
            outcome.newly_derived.len(),
            NetworkIndexAgnosticPath::all_presets().len() * CACHE_FILLING_QUANTITY
        );

        self.profile
            .try_write()
            .unwrap()
            .add_factor_source(factor_source.clone())
            .unwrap();

        Ok(())
    }
}
