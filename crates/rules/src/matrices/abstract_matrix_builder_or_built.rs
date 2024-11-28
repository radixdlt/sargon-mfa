use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AbstractMatrixBuilderOrBuilt<F, T, U> {
    #[serde(skip)]
    #[doc(hidden)]
    pub(crate) built: PhantomData<T>,

    pub(crate) primary_role: AbstractRoleBuilderOrBuilt<F, U>,
    pub(crate) recovery_role: AbstractRoleBuilderOrBuilt<F, U>,
    pub(crate) confirmation_role: AbstractRoleBuilderOrBuilt<F, U>,

    pub(crate) number_of_days_until_auto_confirm: u16,
}
impl<F, T, U> AbstractMatrixBuilderOrBuilt<F, T, U> {
    pub const DEFAULT_NUMBER_OF_DAYS_UNTIL_AUTO_CONFIRM: u16 = 14;
}

pub type AbstractMatrixBuilt<F> = AbstractMatrixBuilderOrBuilt<F, (), ()>;

impl<F: std::cmp::Eq + std::hash::Hash> AbstractMatrixBuilt<F> {
    pub fn all_factors(&self) -> HashSet<&F> {
        let mut factors = HashSet::new();
        factors.extend(self.primary_role.all_factors());
        factors.extend(self.recovery_role.all_factors());
        factors.extend(self.confirmation_role.all_factors());
        factors
    }
}
