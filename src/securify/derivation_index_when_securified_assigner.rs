use crate::prelude::*;

pub trait DerivationIndexWhenSecurifiedAssigner {
    fn assign_derivation_index(
        &self,
        account: Account,
        other_accounts: HashSet<Account>,
    ) -> HDPathComponent;
}
