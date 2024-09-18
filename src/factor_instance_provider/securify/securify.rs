use crate::prelude::*;

impl FactorInstanceProvider {
    pub async fn securify_with_address<E: IsEntity + std::hash::Hash + std::cmp::Eq>(
        &self,
        address: &E::Address,
        matrix: MatrixOfFactorSources,
        profile: &mut Profile,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<SecurifiedEntityControl> {
        let entity = profile.entity_by_address::<E>(address)?;
        self.securify(&entity, &matrix, profile, derivation_interactors)
            .await
    }

    pub async fn securify<E: IsEntity>(
        &self,
        entity: &E,
        matrix: &MatrixOfFactorSources,
        profile: &mut Profile,
        derivation_interactors: Arc<dyn KeysDerivationInteractors>,
    ) -> Result<SecurifiedEntityControl> {
        let entity_kind = E::kind();
        let network_id = entity.address().network_id();
        let key_kind = CAP26KeyKind::TransactionSigning;

        let requests = matrix
            .clone()
            .all_factors()
            .into_iter()
            .map(|factor_source| {
                DerivationRequest::securify(
                    entity_kind,
                    key_kind,
                    factor_source.factor_source_id(),
                    network_id,
                )
            })
            .collect::<IndexSet<_>>();

        let derived_factors_map = self
            .provide_factor_instances(profile, requests, derivation_interactors)
            .await?;

        let derived_factors = derived_factors_map
            .values()
            .cloned()
            .collect::<IndexSet<_>>();

        let matrix = MatrixOfFactorInstances::fulfilling_matrix_of_factor_sources_with_instances(
            derived_factors,
            matrix.clone(),
        )?;

        let component_metadata = ComponentMetadata::new(matrix.clone());

        let securified_entity_control = SecurifiedEntityControl::new(
            matrix,
            AccessController {
                address: AccessControllerAddress::new(entity.entity_address()),
                metadata: component_metadata,
            },
        );

        profile.update_entity(E::new(
            entity.name(),
            entity.entity_address(),
            EntitySecurityState::Securified(securified_entity_control.clone()),
        ));

        let gateway = self.gateway.clone();
        gateway
            .set_securified_entity(securified_entity_control.clone(), entity.address())
            .await?;
        Ok(securified_entity_control)
    }
}
