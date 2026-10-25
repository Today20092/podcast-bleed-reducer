pub use cleanslate_ownership_model::{
    OWNERSHIP_MODEL_RECIPE, OwnershipFitDiagnostics, OwnershipModel, OwnershipPredictor,
    OwnershipTrainingExample, OwnershipTrainingSettings,
};

impl From<cleanslate_ownership_model::RenderBlocker> for crate::RenderBlocker {
    fn from(error: cleanslate_ownership_model::RenderBlocker) -> Self {
        Self(error.0)
    }
}
