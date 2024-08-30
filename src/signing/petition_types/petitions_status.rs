use crate::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PetitionsStatus {
    InProgressNoneInvalid,
    AllAreValid,
    SomeIsInvalid,
}

impl PetitionsStatus {
    pub fn are_all_valid(&self) -> bool {
        matches!(self, Self::AllAreValid)
    }

    pub fn is_some_invalid(&self) -> bool {
        matches!(self, Self::SomeIsInvalid)
    }

    pub(crate) fn reducing(statuses: impl IntoIterator<Item = PetitionFactorsStatus>) -> Self {
        PetitionFactorsStatus::aggregate(
            statuses.into_iter().collect_vec(),
            Self::AllAreValid,
            Self::SomeIsInvalid,
            Self::InProgressNoneInvalid,
        )
    }
}
