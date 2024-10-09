use crate::HasSampleValues;

/// A kind of factor list, either threshold, or override kind.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum FactorListKind {
    Threshold,
    Override,
}

impl HasSampleValues for FactorListKind {
    fn sample() -> Self {
        Self::Threshold
    }

    fn sample_other() -> Self {
        Self::Override
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Sut = FactorListKind;

    #[test]
    fn equality() {
        assert_eq!(Sut::sample(), Sut::sample());
        assert_eq!(Sut::sample_other(), Sut::sample_other());
    }

    #[test]
    fn inequality() {
        assert_ne!(Sut::sample(), Sut::sample_other());
    }
}
