use crate::prelude::*;

/// The reason why a request is unfulfillable, and if the reason is that the
/// last factor would be consumed, the value of that last factor is included,
/// to act as the range.
#[derive(Clone, PartialEq, Eq, Hash, Debug, EnumAsInner)]
pub enum DerivationRequestUnfulfillableByCacheReason {
    /// Users before Radix Wallet 2.0 does not have any cache.
    /// This will be kick of the cumbersome process of analyzing the Profile
    /// and deriving a broad range of keys to find out the "last used" key per
    /// factor source, and then use that to derive the next batch of keys and
    /// cache them.
    Empty,

    /// The request would consume the last factor, the `HDPathComponent` is
    /// the value of this last factor, which we can use as a base for the
    /// next index range to derive keys for, i.e. we will derive keys in the range
    /// `(last_index + 1, last_index + N)` where `N` is the batch size (e.g. 50).
    Last(HDPathComponent),
}
