/// The status of building using a certain list of factors, e.g. threshold or
/// override factors list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PetitionForFactorsStatus {
    /// In progress, still gathering output from factors (signatures or public keys).
    InProgress,

    /// Finished building with factors, either successfully or failed.
    Finished(PetitionFactorsStatusFinished),
}

/// Finished building with factors, either successfully or failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PetitionFactorsStatusFinished {
    /// Successful completion of building with factors.
    Success,

    /// Failure building with factors, either a simulated status, as in what
    /// would happen if we skipped a factor source, or a real failure, as in,
    /// the user explicitly chose to skip a factor source even though she was
    /// advised it would result in some transaction failing. Or we failed to
    /// use a required factor source for what some reason.
    Fail,
}

impl PetitionForFactorsStatus {
    /// Reduces / aggergates a list of `PetitionForFactorsStatus` into some
    /// other status, e.g. `PetitionsStatus`.
    pub fn aggregate<T>(
        statuses: impl IntoIterator<Item = Self>,
        valid: T,
        invalid: T,
        pending: T,
    ) -> T {
        let statuses = statuses.into_iter().collect::<Vec<_>>();

        let are_all_valid = statuses.iter().all(|s| {
            matches!(
                s,
                PetitionForFactorsStatus::Finished(PetitionFactorsStatusFinished::Success)
            )
        });

        if are_all_valid {
            return valid;
        }

        let is_some_invalid = statuses.iter().any(|s| {
            matches!(
                s,
                PetitionForFactorsStatus::Finished(PetitionFactorsStatusFinished::Fail)
            )
        });

        if is_some_invalid {
            return invalid;
        }

        pending
    }
}