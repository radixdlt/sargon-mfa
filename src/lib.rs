#![allow(internal_features)]
#![feature(core_intrinsics)]
#![feature(iter_repeat_n)]
#![feature(async_closure)]
#![allow(unused_imports)]
#![feature(step_trait)]

mod derivation;
mod gateway;
mod samples;
mod signing;
mod types;

#[cfg(test)]
mod testing;

pub mod prelude {
    pub use crate::derivation::*;
    pub use crate::gateway::*;

    pub(crate) use crate::samples::*;

    pub use crate::signing::*;
    pub use crate::types::*;

    #[cfg(test)]
    pub(crate) use crate::testing::*;

    pub(crate) use derive_getters::Getters;
    pub(crate) use enum_as_inner::EnumAsInner;
    pub(crate) use indexmap::{IndexMap, IndexSet};
    pub(crate) use itertools::Itertools;
    pub(crate) use std::cell::RefCell;
    pub(crate) use std::cmp::Ordering;
    pub(crate) use std::future::Future;
    pub(crate) use std::ops::{Deref, DerefMut};
    pub(crate) use std::pin::Pin;
    pub(crate) use std::time::SystemTime;
    pub(crate) use uuid::Uuid;

    pub(crate) use std::ops::Range;

    pub(crate) use sha2::{Digest, Sha256};

    pub(crate) use std::{
        collections::{HashMap, HashSet},
        sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    };

    pub(crate) use log::*;
}

pub use prelude::*;
