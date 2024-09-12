use crate::prelude::*;

impl KeysCollector {
    pub fn securifying<E: IsEntity>(
        entity: &E,
        profile: &Profile,
        matrix: MatrixOfFactorSources,
        index_assigner: impl DerivationIndexWhenSecurifiedAssigner,
        interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<Self> {
        let network_id = entity.network_id();
        let entity_kind = E::kind();
        KeysCollector::new(
            profile.factor_sources.clone(),
            matrix
                .all_factors()
                .clone()
                .into_iter()
                .map(|f| {
                    (
                        f.factor_source_id(),
                        IndexSet::just(DerivationPath::new(
                            network_id,
                            entity_kind,
                            CAP26KeyKind::T9n,
                            index_assigner.derivation_index_for_factor_source(
                                NextFreeIndexAssignerRequest {
                                    key_space: KeySpace::Securified,
                                    entity_kind: CAP26EntityKind::Account,
                                    factor_source_id: FactorSourceIDFromHash::fs0(),
                                    profile,
                                    network_id: NetworkID::Mainnet,
                                },
                            ),
                        )),
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
    profile: &mut Profile,
    derivation_index_assigner: impl DerivationIndexWhenSecurifiedAssigner,
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    gateway: Arc<dyn Gateway>,
) -> Result<SecurifiedEntityControl> {
    let account = profile.account_by_address(address.clone())?;

    let keys_collector = KeysCollector::securifying(
        &account,
        profile,
        matrix.clone(),
        derivation_index_assigner,
        derivation_interactors,
    )?;

    let factor_instances = keys_collector.collect_keys().await.all_factors();

    let matrix = MatrixOfFactorInstances::fulfilling_matrix_of_factor_sources_with_instances(
        factor_instances,
        matrix,
    )?;

    let component_metadata = ComponentMetadata::new(matrix.clone());

    let securified_entity_control = SecurifiedEntityControl::new(
        matrix,
        AccessController {
            address: AccessControllerAddress::new(account.entity_address()),
            metadata: component_metadata,
        },
    );

    profile.update_account(Account::new(
        account.name(),
        account.entity_address(),
        EntitySecurityState::Securified(securified_entity_control.clone()),
    ));

    gateway
        .set_securified_account(securified_entity_control.clone(), &address)
        .await?;
    Ok(securified_entity_control)
}

pub async fn securify(
    address: AccountAddress,
    matrix: MatrixOfFactorSources,
    profile: &mut Profile,
    derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    gateway: Arc<dyn Gateway>,
) -> Result<SecurifiedEntityControl> {
    securify_using(
        address,
        matrix,
        profile,
        NextFreeIndexAssigner::live(),
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

        let b_sec = securify(
            b.entity_address(),
            matrix.clone(),
            &mut profile,
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
