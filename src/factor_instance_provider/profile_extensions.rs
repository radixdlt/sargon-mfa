use crate::prelude::*;

impl Profile {
    async fn new_entity<E: IsEntity + std::fmt::Debug + std::hash::Hash + Eq>(
        &mut self,
        network_id: NetworkID,
        name: impl AsRef<str>,
        factor_source_id: FactorSourceIDFromHash,
        factor_instance_provider: &FactorInstanceProvider,
    ) -> Result<E> {
        assert!(self
            .factor_sources
            .iter()
            .map(|f| f.factor_source_id())
            .contains(&factor_source_id));

        let genesis_factor = factor_instance_provider
            .provide_genesis_factor_for(factor_source_id, E::kind(), network_id, self)
            .await?;

        let address = E::Address::by_hashing(network_id, genesis_factor.clone());

        let entity = E::new(
            name,
            address,
            EntitySecurityState::Unsecured(genesis_factor),
        );

        let erased = Into::<AccountOrPersona>::into(entity.clone());

        match erased {
            AccountOrPersona::AccountEntity(account) => {
                self.accounts.insert(account.entity_address(), account);
            }
            AccountOrPersona::PersonaEntity(persona) => {
                self.personas.insert(persona.entity_address(), persona);
            }
        };

        Ok(entity)
    }

    pub async fn new_account(
        &mut self,
        network_id: NetworkID,
        name: impl AsRef<str>,
        factor_source_id: FactorSourceIDFromHash,
        factor_instance_provider: &FactorInstanceProvider,
    ) -> Result<Account> {
        self.new_entity(network_id, name, factor_source_id, factor_instance_provider)
            .await
    }
}

pub const DERIVATION_INDEX_BATCH_SIZE: HDPathValue = 50;
