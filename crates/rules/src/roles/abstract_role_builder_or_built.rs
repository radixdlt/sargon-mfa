use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AbstractRoleBuilderOrBuilt<F, T> {
    #[serde(skip)]
    #[doc(hidden)]
    built: PhantomData<T>,
    role: RoleKind,
    threshold: u8,
    threshold_factors: Vec<F>,
    override_factors: Vec<F>,
}

pub(crate) type AbstractBuiltRoleWithFactor<F> = AbstractRoleBuilderOrBuilt<F, ()>;
pub(crate) type RoleBuilder = AbstractRoleBuilderOrBuilt<FactorSourceID, RoleWithFactorSourceIds>;

impl<F: IsMaybeKeySpaceAware, T> AbstractRoleBuilderOrBuilt<F, T> {
    pub(crate) fn with_factors(
        role: RoleKind,
        threshold: u8,
        threshold_factors: impl IntoIterator<Item = F>,
        override_factors: impl IntoIterator<Item = F>,
    ) -> Self {
        let assert_is_securified = |factors: &Vec<F>| -> Result<(), CommonError> {
            let trait_objects: Vec<&dyn IsMaybeKeySpaceAware> = factors
                .iter()
                .map(|x| x as &dyn IsMaybeKeySpaceAware)
                .collect();
            if trait_objects
                .iter()
                .filter_map(|x| x.maybe_key_space())
                .any(|x| x != KeySpace::Securified)
            {
                return Err(crate::CommonError::IndexUnsecurifiedExpectedSecurified);
            }
            Ok(())
        };

        let threshold_factors = threshold_factors.into_iter().collect();
        let override_factors = override_factors.into_iter().collect();

        assert_is_securified(&threshold_factors)
            .expect("Should not have allowed building of invalid Role");
        assert_is_securified(&override_factors)
            .expect("Should not have allowed building of invalid Role");

        Self {
            built: PhantomData,
            role,
            threshold,
            threshold_factors,
            override_factors,
        }
    }
}

impl<F, T> AbstractRoleBuilderOrBuilt<F, T> {
    pub fn all_factors(&self) -> Vec<&F> {
        self.threshold_factors
            .iter()
            .chain(self.override_factors.iter())
            .collect()
    }

    pub fn get_threshold_factors(&self) -> &Vec<F> {
        &self.threshold_factors
    }

    pub fn get_override_factors(&self) -> &Vec<F> {
        &self.override_factors
    }

    pub fn get_threshold(&self) -> u8 {
        self.threshold
    }
}

impl RoleBuilder {
    pub(crate) fn new(role: RoleKind) -> Self {
        Self {
            built: PhantomData,
            role,
            threshold: 0,
            threshold_factors: Vec::new(),
            override_factors: Vec::new(),
        }
    }

    pub(crate) fn role(&self) -> RoleKind {
        self.role
    }

    pub(crate) fn mut_threshold_factors(&mut self) -> &mut Vec<FactorSourceID> {
        &mut self.threshold_factors
    }

    pub(crate) fn mut_override_factors(&mut self) -> &mut Vec<FactorSourceID> {
        &mut self.override_factors
    }

    pub(crate) fn unchecked_add_factor_source_to_list(
        &mut self,
        factor_source_id: FactorSourceID,
        factor_list_kind: FactorListKind,
    ) {
        match factor_list_kind {
            FactorListKind::Threshold => self.threshold_factors.push(factor_source_id),
            FactorListKind::Override => self.override_factors.push(factor_source_id),
        }
    }

    pub(crate) fn unchecked_set_threshold(&mut self, threshold: u8) {
        self.threshold = threshold;
    }
}

impl<F> AbstractBuiltRoleWithFactor<F> {
    pub fn role(&self) -> RoleKind {
        self.role
    }

    pub fn threshold(&self) -> u8 {
        self.threshold
    }

    pub fn threshold_factors(&self) -> &Vec<F> {
        &self.threshold_factors
    }

    pub fn override_factors(&self) -> &Vec<F> {
        &self.override_factors
    }
}
