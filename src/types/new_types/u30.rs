use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, derive_more::Display, Ord, Hash)]
#[display("{inner}")]
pub struct U30 {
    hidden_constructor: HiddenConstructor,
    pub inner: u32,
}

impl U30 {
    pub const MAX: u32 = u32::MAX / 4;
    pub fn new(inner: u32) -> Result<Self> {
        if inner > Self::MAX {
            return Err(CommonError::Invalid30 { bad_value: inner });
        }
        Ok(Self {
            hidden_constructor: HiddenConstructor,
            inner,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[allow(clippy::upper_case_acronyms)]
    type SUT = U30;

    #[test]
    fn invalid_too_large() {
        assert_eq!(
            SUT::new(U30::MAX + 1),
            Err(CommonError::Invalid30 {
                bad_value: U30::MAX + 1
            })
        );
    }

    #[test]
    fn valid_max() {
        assert!(SUT::new(U30::MAX).is_ok())
    }

    #[test]
    fn inner() {
        assert_eq!(SUT::new(1024).unwrap().inner, 1024);
    }

    #[test]
    fn ord() {
        assert!(SUT::new(0).unwrap() < SUT::new(1).unwrap());
        assert!(SUT::new(U30::MAX).unwrap() > SUT::new(U30::MAX - 1).unwrap());
    }
}
