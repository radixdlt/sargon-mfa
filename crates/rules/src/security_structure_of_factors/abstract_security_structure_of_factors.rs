use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractSecurityStructure<F> {
    /// Metadata of this Security Structure, such as globally unique and
    /// stable identifier, creation date and user chosen label (name).
    pub metadata: sargon::SecurityStructureMetadata,

    /// The structure of factors to use for certain roles, Primary, Recovery
    /// and Confirmation role.
    pub matrix_of_factors: AbstractMatrixBuilt<F>,
}

impl<F> AbstractSecurityStructure<F> {
    pub fn with_metadata(
        metadata: sargon::SecurityStructureMetadata,
        matrix_of_factors: AbstractMatrixBuilt<F>,
    ) -> Self {
        Self {
            metadata,
            matrix_of_factors,
        }
    }

    pub fn new(display_name: DisplayName, matrix_of_factors: AbstractMatrixBuilt<F>) -> Self {
        let metadata = sargon::SecurityStructureMetadata::new(display_name);
        Self::with_metadata(metadata, matrix_of_factors)
    }
}
