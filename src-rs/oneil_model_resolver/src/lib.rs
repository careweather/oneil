#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! Model resolver for the Oneil programming language

use std::path::Path;

use indexmap::IndexMap;
use oneil_ir as ir;

pub mod error;
mod resolver;
mod util;

#[cfg(test)]
mod test;

pub use crate::error::{CircularDependencyError, ResolutionErrors};
pub use crate::util::{
    AstLoadingFailedError, ExternalResolutionContext, ModelResolutionResult,
    PythonImportLoadingFailedError,
};

use crate::util::ResolutionContext;

/// Result of loading one or more models: resolved models and any per-model errors.
#[derive(Debug)]
pub struct LoadModelResult {
    /// Resolved models by path.
    pub models: IndexMap<ir::ModelPath, ir::Model>,
    /// Per-model resolution errors (import, reference, submodel, parameter, test).
    pub model_errors: IndexMap<ir::ModelPath, ResolutionErrors>,
    /// Per-model circular dependency errors.
    pub circular_dependency_errors: IndexMap<ir::ModelPath, Vec<CircularDependencyError>>,
}

/// Loads a single model and all its dependencies.
///
/// Returns the resolved models, per-model resolution errors, and circular dependency errors.
pub fn load_model<E>(
    model_path: impl AsRef<Path>,
    external_context: &mut E,
) -> ModelResolutionResult
where
    E: ExternalResolutionContext,
{
    load_model_list(&[model_path], external_context)
}

/// Loads multiple models and all their dependencies.
///
/// Returns the resolved models, per-model resolution errors, and circular dependency errors.
pub fn load_model_list<E>(
    model_paths: &[impl AsRef<Path>],
    external_context: &mut E,
) -> ModelResolutionResult
where
    E: ExternalResolutionContext,
{
    let mut resolution_context = ResolutionContext::new(external_context);

    for model_path in model_paths {
        let model_path = ir::ModelPath::new(model_path);
        resolver::load_model(&model_path, &mut resolution_context);
    }

    resolution_context.into_result()
}
