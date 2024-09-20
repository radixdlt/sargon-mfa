use crate::prelude::*;

#[derive(Debug)]
pub struct AccountRecoveryScanOutcome {
    pub cache: PreDerivedKeysCache,
    pub profile: Profile,
}

impl From<DerivationAndAnalysisAccountRecoveryScan> for AccountRecoveryScanOutcome {
    fn from(value: DerivationAndAnalysisAccountRecoveryScan) -> Self {
        let profile = Profile::new(
            value.factor_sources(),
            value.recovered_accounts().as_slice(),
            [],
        );
        let cache = PreDerivedKeysCache::new(value.probably_free_instances);
        Self { profile, cache }
    }
}
