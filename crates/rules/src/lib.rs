mod rules;

pub mod prelude {

    pub(crate) use crate::rules::*;

    pub(crate) use thiserror::Error as ThisError;
}

pub use prelude::*;
