use crate::prelude::*;

impl NeglectedFactorInstance {
    pub fn as_neglected_factor(&self) -> NeglectedFactor {
        NeglectedFactor::new(self.reason, self.factor_source_id())
    }
}
impl FactorSourceReferencing for NeglectedFactorInstance {
    fn factor_source_id(&self) -> FactorSourceIDFromHash {
        self.content.factor_source_id()
    }
}

impl FactorSourceReferencing for NeglectedFactor {
    fn factor_source_id(&self) -> FactorSourceIDFromHash {
        self.content
    }
}

impl HasSampleValues for NeglectedFactorInstance {
    fn sample() -> Self {
        Self::new(
            NeglectFactorReason::UserExplicitlySkipped,
            HierarchicalDeterministicFactorInstance::sample(),
        )
    }
    fn sample_other() -> Self {
        Self::new(
            NeglectFactorReason::Failure,
            HierarchicalDeterministicFactorInstance::sample_other(),
        )
    }
}

pub type NeglectedFactor = AbstractNeglectedFactor<FactorSourceIDFromHash>;
pub type NeglectedFactors = AbstractNeglectedFactor<IndexSet<FactorSourceIDFromHash>>;
pub type NeglectedFactorInstance = AbstractNeglectedFactor<HierarchicalDeterministicFactorInstance>;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AbstractNeglectedFactor<T> {
    pub reason: NeglectFactorReason,
    pub content: T,
}
impl<T> AbstractNeglectedFactor<T> {
    pub fn new(reason: NeglectFactorReason, content: T) -> Self {
        Self { reason, content }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for AbstractNeglectedFactor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Neglected")
            .field("reason", &self.reason)
            .field("content", &self.content)
            .finish()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, derive_more::Debug, derive_more::Display)]
pub enum NeglectFactorReason {
    #[display("User Skipped")]
    #[debug("UserExplicitlySkipped")]
    UserExplicitlySkipped,

    #[display("Failure")]
    #[debug("Failure")]
    Failure,
}
