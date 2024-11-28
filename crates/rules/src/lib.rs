mod matrices;
mod move_to_sargon;
mod roles;

pub mod prelude {
    pub(crate) use sargon::{
        CommonError, FactorInstance, FactorSource, FactorSourceID, FactorSourceIDFromHash,
        FactorSourceKind, FactorSources, HasSampleValues, Identifiable, IndexSet, RoleKind,
    };

    #[allow(unused_imports)]
    pub use crate::matrices::*;
    pub use crate::move_to_sargon::*;
    pub use crate::roles::*;

    pub(crate) use serde::{Deserialize, Serialize};
    pub(crate) use std::collections::HashSet;
    pub(crate) use std::marker::PhantomData;
}

pub use crate::prelude::*;
