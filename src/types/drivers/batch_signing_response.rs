use crate::prelude::*;

/// The response of a batch signing request, either a Parallel or Serial signing
/// request, matters not, because the goal is to have signed all transactions with
/// enough keys (derivation paths) needed for it to be valid when submitted to the
/// Radix network.
#[derive(Clone, PartialEq, Eq, derive_more::Debug)]
#[debug("BatchSigningResponse {{ signatures: {:?} }}", signatures.values().into_iter().map(|f| format!("{:?}", f)).join(", "))]
pub struct BatchSigningResponse {
    pub signatures: IndexMap<FactorSourceID, IndexSet<HDSignature>>,
}
impl BatchSigningResponse {
    pub fn new(signatures: IndexMap<FactorSourceID, IndexSet<HDSignature>>) -> Self {
        Self { signatures }
    }
}
