#![allow(internal_features)]
#![feature(core_intrinsics)]
#![feature(iter_repeat_n)]
#![feature(async_closure)]

mod derivation;
mod samples;
mod signing;
mod types;

#[cfg(test)]
mod testing;

pub mod prelude {
    pub use crate::derivation::*;

    #[allow(unused_imports)]
    pub(crate) use crate::samples::*;
    pub use crate::signing::*;
    pub use crate::types::*;

    #[cfg(test)]
    pub(crate) use crate::testing::*;

    pub(crate) use derive_getters::Getters;
    pub(crate) use indexmap::{IndexMap, IndexSet};
    pub(crate) use itertools::Itertools;
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
