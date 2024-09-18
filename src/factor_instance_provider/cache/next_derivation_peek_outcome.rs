use crate::prelude::*;

/// The outcome of peeking the next derivation index for a request.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NextDerivationPeekOutcome {
    /// We failed to peek the next derivation index for the request, probably
    /// an error while reading from cache.
    Failure(CommonError),

    /// All requests would have at least one factor left after they are fulfilled.
    Fulfillable,

    /// The `IndexMap` contains the last consumed index for each request.
    ///
    /// N.B. that if some request would not consume the last factor, it will not
    /// be present in this map.
    Unfulfillable(UnfulfillableRequests),
}
