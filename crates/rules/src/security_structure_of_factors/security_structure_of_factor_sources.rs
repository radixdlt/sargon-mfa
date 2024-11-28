use crate::prelude::*;

pub type SecurityStructureOfFactorSources = AbstractSecurityStructure<FactorSource>;

impl HasSampleValues for SecurityStructureOfFactorSources {
    fn sample() -> Self {
        let metadata = sargon::SecurityStructureMetadata::sample();
        Self::with_metadata(metadata, MatrixWithFactorSources::sample())
    }

    fn sample_other() -> Self {
        let metadata = sargon::SecurityStructureMetadata::sample_other();
        Self::with_metadata(metadata, MatrixWithFactorSources::sample_other())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(clippy::upper_case_acronyms)]
    type SUT = SecurityStructureOfFactorSources;

    #[test]
    fn equality() {
        assert_eq!(SUT::sample(), SUT::sample());
        assert_eq!(SUT::sample_other(), SUT::sample_other());
    }

    #[test]
    fn inequality() {
        assert_ne!(SUT::sample(), SUT::sample_other());
    }
}
