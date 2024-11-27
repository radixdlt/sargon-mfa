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
