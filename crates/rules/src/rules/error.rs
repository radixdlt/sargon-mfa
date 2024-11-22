use crate::prelude::*;


#[derive(Clone, Copy, Debug, PartialEq, Eq, ThisError)]
pub enum Error {
    #[error("Unknown")]
    Unknown,
}