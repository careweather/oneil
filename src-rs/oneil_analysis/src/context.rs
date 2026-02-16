//! Context types for tree traversal and analysis.

use std::path::PathBuf;

use indexmap::IndexMap;
use oneil_ir as ir;

/// External context provided to tree operations.
pub trait ExternalTreeContext {
    /// Returns the full map of model paths to their IR models.
    fn get_all_model_ir(&self) -> IndexMap<&PathBuf, &ir::Model>;
}

/// Context for tree operations that holds a mutable reference to an [`ExternalTreeContext`].
pub struct TreeContext<'external, E: ExternalTreeContext> {
    /// Mutable reference to the external tree context.
    external: &'external mut E,
}

impl<'external, E: ExternalTreeContext> TreeContext<'external, E> {
    /// Creates a new tree context with the given mutable reference to an external context.
    #[must_use]
    pub const fn new(external: &'external mut E) -> Self {
        Self { external }
    }

    /// Returns the full map of model paths to their IR models, delegating to the external context.
    #[must_use]
    pub fn get_all_model_ir(&self) -> IndexMap<&PathBuf, &ir::Model> {
        self.external.get_all_model_ir()
    }
}
