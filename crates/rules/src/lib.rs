mod matrices;
mod move_to_sargon;
mod roles;
mod security_structure_of_factors;

pub mod prelude {
    pub(crate) use sargon::{
        BaseIsFactorSource, CommonError, DisplayName, FactorInstance, FactorInstances,
        FactorSource, FactorSourceID, FactorSourceIDFromHash, FactorSourceKind, FactorSources,
        HasSampleValues, HierarchicalDeterministicFactorInstance, Identifiable, IndexMap, IndexSet,
        IsMaybeKeySpaceAware, KeySpace, RoleKind,
    };

    #[cfg(test)]
    pub(crate) use sargon::JustKV;

    #[allow(unused_imports)]
    pub use crate::matrices::*;
    pub use crate::move_to_sargon::*;
    pub use crate::roles::*;
    pub use crate::security_structure_of_factors::*;

    pub(crate) use serde::{Deserialize, Serialize};
    pub(crate) use std::collections::HashSet;
    pub(crate) use std::marker::PhantomData;
}

pub use crate::prelude::*;
