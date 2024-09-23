use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FactorInstanceInUnsecurifiedSpace {
    factor_instance: HierarchicalDeterministicFactorInstance,
}
impl From<FactorInstanceInUnsecurifiedSpace> for HierarchicalDeterministicFactorInstance {
    fn from(value: FactorInstanceInUnsecurifiedSpace) -> Self {
        value.instance()
    }
}
impl FactorInstanceInUnsecurifiedSpace {
    /// # Panics
    /// Panics if it is not in unsecurified space
    pub fn new(factor_instance: HierarchicalDeterministicFactorInstance) -> Self {
        assert_eq!(factor_instance.key_space(), KeySpace::Unsecurified);
        Self { factor_instance }
    }
    pub fn instance(&self) -> HierarchicalDeterministicFactorInstance {
        self.factor_instance.clone()
    }
}
