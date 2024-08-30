mod derivation;
mod signing;

pub(crate) use derivation::*;
pub(crate) use signing::*;

#[cfg(test)]
mod common_tests {

    use crate::prelude::*;

    #[test]
    fn factors_sources() {
        assert_eq!(ALL_FACTOR_SOURCES.clone(), ALL_FACTOR_SOURCES.clone());
    }

    #[test]
    fn factors_source_ids() {
        assert_eq!(FactorSourceIDFromHash::fs0(), FactorSourceIDFromHash::fs0());
        assert_eq!(FactorSourceIDFromHash::fs1(), FactorSourceIDFromHash::fs1());
        assert_ne!(FactorSourceIDFromHash::fs0(), FactorSourceIDFromHash::fs1());
    }

    #[test]
    fn factor_instance_in_accounts() {
        assert_eq!(
            Account::a0().security_state.all_factor_instances(),
            Account::a0().security_state.all_factor_instances()
        );
        assert_eq!(
            Account::a6().security_state.all_factor_instances(),
            Account::a6().security_state.all_factor_instances()
        );
    }
}
