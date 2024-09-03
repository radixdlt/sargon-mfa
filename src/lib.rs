#![allow(internal_features)]
#![feature(core_intrinsics)]
#![feature(iter_repeat_n)]
#![feature(async_closure)]
#![allow(unused_imports)]

mod derivation;
mod recovery;
mod samples;
mod securify;
mod signing;
mod types;

#[cfg(test)]
mod testing;

pub mod prelude {
    pub use crate::derivation::*;

    #[cfg(test)]
    pub use crate::recovery::*;

    pub(crate) use crate::samples::*;

    #[cfg(test)]
    pub use crate::securify::*;

    pub use crate::signing::*;
    pub use crate::types::*;

    #[cfg(test)]
    pub(crate) use crate::testing::*;

    pub use core::borrow::Borrow;
    pub(crate) use derive_getters::Getters;
    pub(crate) use indexmap::{IndexMap, IndexSet};
    pub(crate) use itertools::Itertools;
    use sha2::{Digest, Sha256};
    pub(crate) use std::cell::RefCell;
    pub(crate) use std::time::SystemTime;
    pub(crate) use uuid::Uuid;

    pub(crate) use std::{
        collections::{HashMap, HashSet},
        sync::Arc,
    };

    pub(crate) use log::*;
}

pub use prelude::*;
