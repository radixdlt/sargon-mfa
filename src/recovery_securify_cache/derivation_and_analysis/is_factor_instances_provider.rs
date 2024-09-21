use crate::prelude::*;

#[async_trait::async_trait]
pub trait IsFactorInstancesProvider: Sync + Send {
    async fn provide_instances(
        &self,
        derivation_requests: DerivationRequests,
    ) -> Result<FactorInstances>;
}
