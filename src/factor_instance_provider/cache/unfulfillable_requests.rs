use crate::prelude::*;

/// A non-empty collection of unfulfillable requests
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DerivationRequestsUnfulfillableByCache {
    /// A non-empty collection of unfulfillable requests
    unfulfillable: Vec<DerivationRequestUnfulfillableByCache>, // we want `Set` but `IndexSet` is not `Hash`
}
impl DerivationRequestsUnfulfillableByCache {
    /// # Panics
    /// Panics if `unfulfillable` is empty.
    pub fn new(unfulfillable: IndexSet<DerivationRequestUnfulfillableByCache>) -> Self {
        assert!(!unfulfillable.is_empty(), "non_empty must not be empty");
        Self {
            unfulfillable: unfulfillable.into_iter().collect(),
        }
    }
    pub fn unfulfillable(&self) -> IndexSet<DerivationRequestUnfulfillableByCache> {
        self.unfulfillable.clone().into_iter().collect()
    }

    pub fn requests(&self) -> IndexSet<DerivationRequest> {
        self.unfulfillable
            .clone()
            .into_iter()
            .map(|ur| ur.request)
            .collect()
    }
}
