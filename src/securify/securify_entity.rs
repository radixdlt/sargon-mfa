use crate::{factor_instance_provider, prelude::*};

pub async fn securify(
    address: AccountAddress,
    matrix: MatrixOfFactorSources,
    profile: &mut Profile,
    factor_instance_provider: &FactorInstanceProvider,
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    gateway: Arc<dyn Gateway>,
) -> Result<SecurifiedEntityControl> {
    let account = profile.account_by_address(address.clone())?;

    // let keys_collector = KeysCollector::securifying(
    //     &account,
    //     profile,
    //     matrix.clone(),
    //     derivation_index_assigner,
    //     derivation_interactors,
    // )?;

    // let factor_instances = keys_collector.collect_keys().await.all_factors();

    todo!()

    // let matrix = MatrixOfFactorInstances::fulfilling_matrix_of_factor_sources_with_instances(
    //     factor_instances,
    //     matrix,
    // )?;

    // let component_metadata = ComponentMetadata::new(matrix.clone());

    // let securified_entity_control = SecurifiedEntityControl::new(
    //     matrix,
    //     AccessController {
    //         address: AccessControllerAddress::new(account.entity_address()),
    //         metadata: component_metadata,
    //     },
    // );

    // profile.update_account(Account::new(
    //     account.name(),
    //     account.entity_address(),
    //     EntitySecurityState::Securified(securified_entity_control.clone()),
    // ));

    // gateway
    //     .set_securified_account(securified_entity_control.clone(), &address)
    //     .await?;
    // Ok(securified_entity_control)
}

#[cfg(test)]
mod securify_tests {

    use super::*;

    #[actix_rt::test]
    async fn derivation_path_is_never_same_after_securified() {
        let all_factors = HDFactorSource::all();
        let a = &Account::unsecurified_mainnet(
            "A0",
            HierarchicalDeterministicFactorInstance::mainnet_tx(
                CAP26EntityKind::Account,
                HDPathComponent::unsecurified(0),
                FactorSourceIDFromHash::fs0(),
            ),
        );
        let b = &Account::unsecurified_mainnet(
            "A1",
            HierarchicalDeterministicFactorInstance::mainnet_tx(
                CAP26EntityKind::Account,
                HDPathComponent::unsecurified(1),
                FactorSourceIDFromHash::fs0(),
            ),
        );

        let mut profile = Profile::new(all_factors.clone(), [a, b], []);
        let matrix = MatrixOfFactorSources::new([fs_at(0)], 1, []);

        let interactors = Arc::new(TestDerivationInteractors::default());
        let gateway = Arc::new(TestGateway::default());

        let factor_instance_provider = FactorInstanceProvider::new(
            gateway.clone(),
            Arc::new(InMemoryPreDerivedKeysCache::default()),
        );

        let b_sec = securify(
            b.entity_address(),
            matrix.clone(),
            &mut profile,
            &factor_instance_provider,
            interactors.clone(),
            gateway.clone(),
        )
        .await
        .unwrap();

        assert_eq!(
            b_sec
                .matrix
                .all_factors()
                .clone()
                .into_iter()
                .map(|f| f.derivation_path().index)
                .collect::<HashSet<_>>(),
            HashSet::just(HDPathComponent::securified(0))
        );

        let a_sec = securify(
            a.entity_address(),
            matrix.clone(),
            &mut profile,
            &factor_instance_provider,
            interactors.clone(),
            gateway.clone(),
        )
        .await
        .unwrap();

        assert_eq!(
            a_sec
                .matrix
                .all_factors()
                .clone()
                .into_iter()
                .map(|f| f.derivation_path().index)
                .collect::<HashSet<_>>(),
            HashSet::just(HDPathComponent::securified(1))
        );
    }
}
