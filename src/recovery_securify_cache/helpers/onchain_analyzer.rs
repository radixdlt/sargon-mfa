#![allow(unused)]

use crate::prelude::*;
#[derive(Default, Clone)]
pub struct OnChainAnalyzer {
    gateway: Option<Arc<dyn Gateway>>,
}
impl OnChainAnalyzer {
    pub fn new(gateway: impl Into<Option<Arc<dyn Gateway>>>) -> Self {
        Self {
            gateway: gateway.into(),
        }
    }

    pub fn with_gateway(gateway: Arc<dyn Gateway>) -> Self {
        Self::new(gateway)
    }

    pub fn dummy() -> Self {
        Self::new(None)
    }
}
