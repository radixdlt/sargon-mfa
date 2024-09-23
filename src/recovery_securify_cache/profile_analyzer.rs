#![allow(unused)]

use crate::prelude::*;

#[derive(Default, Clone)]
pub struct ProfileAnalyzer {
    profile: Option<Arc<Profile>>,
}
impl ProfileAnalyzer {
    fn new(profile: impl Into<Option<Arc<Profile>>>) -> Self {
        Self {
            profile: profile.into(),
        }
    }

    pub fn with_profile(profile: Arc<Profile>) -> Self {
        Self::new(profile)
    }

    pub fn dummy() -> Self {
        Self::new(None)
    }
}
