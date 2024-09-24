#![allow(unused)]

use crate::prelude::*;
#[derive(Default, Clone)]
pub struct OnChainKeyStatusAnalyzer {
    gateway: Option<Arc<dyn GatewayReadonly>>,
}
impl OnChainKeyStatusAnalyzer {
    pub fn new(gateway: impl Into<Option<Arc<dyn GatewayReadonly>>>) -> Self {
        Self {
            gateway: gateway.into(),
        }
    }

    pub fn with_gateway(gateway: Arc<dyn GatewayReadonly>) -> Self {
        Self::new(gateway)
    }

    pub fn dummy() -> Self {
        Self::new(None)
    }
}
