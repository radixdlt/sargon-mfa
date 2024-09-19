use crate::prelude::*;

#[async_trait::async_trait]
pub trait Gateway: GatewayReadonly {
    async fn simulate_network_activity_for(&self, owner: AddressOfAccountOrPersona) -> Result<()>;

    async fn set_securified_entity(
        &self,
        securified: SecurifiedEntityControl,
        owner: AddressOfAccountOrPersona,
    ) -> Result<()>;

    async fn set_securified_account(
        &self,
        securified: SecurifiedEntityControl,
        owner: &AccountAddress,
    ) -> Result<()> {
        self.set_securified_entity(securified, owner.clone().into())
            .await
    }

    async fn set_securified_persona(
        &self,
        securified: SecurifiedEntityControl,
        owner: &IdentityAddress,
    ) -> Result<()> {
        self.set_securified_entity(securified, owner.clone().into())
            .await
    }
}
