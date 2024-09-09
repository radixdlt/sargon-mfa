use crate::prelude::*;

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
    gateway: Arc<dyn Gateway>,
) -> Result<SecurifiedEntityControl> {
    let account = profile.account_by_address(address.clone())?;
    let network_id = account.network_id();

    let derivation_index = derivation_index_assigner.assign_derivation_index(profile, network_id);
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

    let securified_entity_control = SecurifiedEntityControl::new(
        matrix,
        AccessController {
            address: AccessControllerAddress::generate(),
            metadata: component_metadata,
        },
    );

    gateway
        .set_securified_account(securified_entity_control.clone(), &address)
        .await?;
    Ok(securified_entity_control)
}

pub async fn securify(
    address: AccountAddress,
    matrix: MatrixOfFactorSources,
    profile: &Profile,
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    gateway: Arc<dyn Gateway>,
) -> Result<SecurifiedEntityControl> {
    securify_using(
        address,
        matrix,
        profile,
        CanonicalEntityIndexingNextFreeIndexAssigner::live(),
        derivation_interactors,
        gateway,
    )
    .await
}

#[cfg(test)]
mod securify_tests {

    use super::*;

    #[actix_rt::test]
    async fn derivation_path_is_never_same_after_securified() {
        let all_factors = HDFactorSource::all();
        let a = &Account::unsecurified_mainnet(0, "A", FactorSourceIDFromHash::fs0());
        let b = &Account::unsecurified_mainnet(1, "B", FactorSourceIDFromHash::fs0());

        let profile = Profile::new(all_factors.clone(), [a, b], []);
        let matrix = MatrixOfFactorSources::new([fs_at(0)], 1, []);

        let interactors = Arc::new(TestDerivationInteractors::default());
        let gateway = Arc::new(TestGateway::default());

        let b_sec = securify(
            b.entity_address(),
            matrix.clone(),
            &profile,
            interactors.clone(),
            gateway.clone(),
        )
        .await
        .unwrap();

        assert_eq!(
            b_sec.access_controller.metadata.derivation_index,
            HDPathComponent::securified(0)
        );

        // uh update profile... since we dont have proper Profile impl in this repo.
        let profile = Profile::new(
            all_factors,
            [
                a,
                &Account::new("B", EntitySecurityState::Securified(b_sec)),
            ],
            [],
        );
        let a_sec = securify(
            a.entity_address(),
            matrix.clone(),
            &profile,
            interactors.clone(),
            gateway.clone(),
        )
        .await
        .unwrap();

        assert_eq!(
            a_sec.access_controller.metadata.derivation_index,
            HDPathComponent::securified(1)
        );
    }
}
