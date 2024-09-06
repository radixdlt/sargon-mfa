use crate::prelude::*;

impl Profile {
    fn account_and_others(&self, address: AccountAddress) -> Result<(Account, HashSet<Account>)> {
        let account = self.account_by_address(address)?;

        let mut other_accounts = self
            .accounts
            .values()
            .cloned()
            .collect::<HashSet<Account>>();

        other_accounts.remove(&account);

        Ok((account, other_accounts))
    }
}

impl KeysCollector {
    pub fn securifying(
        matrix: MatrixOfFactorSources,
        derivation_path: DerivationPath,
        profile: &Profile,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<Self> {
        KeysCollector::new(
            profile.factor_sources.clone(),
            matrix
                .all_factors()
                .clone()
                .into_iter()
                .map(|f| {
                    (
                        f.factor_source_id(),
                        IndexSet::just(derivation_path.clone()),
                    )
                })
                .collect::<IndexMap<FactorSourceIDFromHash, IndexSet<DerivationPath>>>(),
            interactors,
        )
    }
}

async fn securify_using(
    address: AccountAddress,
    matrix: MatrixOfFactorSources,
    profile: &Profile,
    derivation_index_assigner: impl DerivationIndexWhenSecurifiedAssigner,
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
) -> Result<SecurifiedEntityControl> {
    let (account, other_accounts) = profile.account_and_others(address)?;

    let derivation_index =
        derivation_index_assigner.assign_derivation_index(account, other_accounts);
    let derivation_path = DerivationPath::account_tx(NetworkID::Mainnet, derivation_index);

    let keys_collector = KeysCollector::securifying(
        matrix.clone(),
        derivation_path,
        profile,
        derivation_interactors,
    )?;

    let factor_instances = keys_collector.collect_keys().await.all_factors();

    let matrix = MatrixOfFactorInstances::fulfilling_matrix_of_factor_sources_with_instances(
        factor_instances,
        matrix,
    )?;

    let component_metadata = ComponentMetadata::new(matrix.all_factors(), derivation_index);

    Ok(SecurifiedEntityControl::new(
        matrix,
        AccessController {
            address: AccessControllerAddress::generate(),
            metadata: component_metadata,
        },
    ))
}

pub async fn securify(
    address: AccountAddress,
    matrix: MatrixOfFactorSources,
    profile: &Profile,
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
) -> Result<SecurifiedEntityControl> {
    securify_using(
        address,
        matrix,
        profile,
        RandomFreeIndexAssigner::live(),
        derivation_interactors,
    )
    .await
}

#[cfg(test)]
mod securify_tests {

    use super::*;

    #[actix_rt::test]
    async fn derivation_path_is_never_same_after_securified() {
        let all_factors = HDFactorSource::all();
        let account = Account::sample_unsecurified();
        let profile = Profile::new(all_factors, [&account], []);
        let matrix = MatrixOfFactorSources::new([fs_at(0)], 1, []);

        let securified = securify(
            account.entity_address(),
            matrix,
            &profile,
            Arc::new(TestDerivationInteractors::default()),
        )
        .await
        .unwrap();

        assert_ne!(
            securified
                .access_controller
                .metadata
                .derivation_index
                .index(),
            0
        );

        assert_eq!(
            securified
                .matrix
                .all_factors()
                .into_iter()
                .map(|fi| fi.derivation_path())
                .collect::<HashSet<_>>(),
            HashSet::just(DerivationPath::new(
                NetworkID::Mainnet,
                CAP26EntityKind::Account,
                CAP26KeyKind::T9n,
                securified.access_controller.metadata.derivation_index
            ))
        );
    }
}
