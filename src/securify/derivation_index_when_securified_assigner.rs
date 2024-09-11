use crate::prelude::*;

pub trait DerivationIndexWhenSecurifiedAssigner {
    fn derivation_index_for_factor_source(
        &self,
        request: NextFreeIndexAssignerRequest,
    ) -> HDPathComponent;
}
