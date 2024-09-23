use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FactorInstanceInSecurifiedSpace {
    factor_instance: HierarchicalDeterministicFactorInstance,
}
impl From<FactorInstanceInSecurifiedSpace> for HierarchicalDeterministicFactorInstance {
    fn from(value: FactorInstanceInSecurifiedSpace) -> Self {
        value.instance()
    }
}
impl FactorInstanceInSecurifiedSpace {
    /// # Panics
    /// Panics if it is not in securified space
    pub fn new(factor_instance: HierarchicalDeterministicFactorInstance) -> Self {
        assert_eq!(factor_instance.key_space(), KeySpace::Securified);
        Self { factor_instance }
    }
    pub fn instance(&self) -> HierarchicalDeterministicFactorInstance {
        self.factor_instance.clone()
    }
}
