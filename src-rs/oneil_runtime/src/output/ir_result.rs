//! IR load result type.
//!
//! [`IrLoadResult`] represents the outcome of loading IR for a model and its
//! dependencies. On success it holds a reference to the resolved model IR; on
//! failure it holds an [`IrLoadError`] with resolution and Python import errors.

use oneil_shared::error::OneilError;

use super::reference::{ModelIrReference, ResolutionErrorReference};

/// Result of loading IR for a model and its dependencies.
///
/// On success, holds a reference to the resolved model IR. On failure, holds an
/// [`IrLoadError`] with a reference to the resolution error and any Python import
/// errors that occurred.
#[derive(Debug)]
pub struct IrLoadResult<'runtime>(Result<ModelIrReference<'runtime>, IrLoadError<'runtime>>);

impl<'runtime> IrLoadResult<'runtime> {
    /// Builds a successful result from a reference to the resolved model IR.
    #[must_use]
    pub const fn ok(model_ref: ModelIrReference<'runtime>) -> Self {
        Self(Ok(model_ref))
    }

    /// Builds a failure result from a resolution error reference and a list of
    /// Python import errors.
    #[must_use]
    pub const fn err(
        error: ResolutionErrorReference<'runtime>,
        python_import_errors: Vec<OneilError>,
    ) -> Self {
        Self(Err(IrLoadError::new(error, python_import_errors)))
    }

    /// Returns a reference to the model IR if loading succeeded, or to the partial
    /// IR (if any) when resolution failed.
    #[must_use]
    pub fn maybe_partial_ir(&self) -> Option<ModelIrReference<'runtime>> {
        match &self.0 {
            Ok(model_ref) => Some(*model_ref),
            Err(error) => error.resolution_error.partial_ir(),
        }
    }
}

impl<'runtime> From<IrLoadResult<'runtime>>
    for Result<ModelIrReference<'runtime>, IrLoadError<'runtime>>
{
    fn from(result: IrLoadResult<'runtime>) -> Self {
        result.0
    }
}

/// Error produced when loading IR for a model fails.
///
/// Contains a reference to the resolution error (parse or resolution failures)
/// and any Python import errors that occurred while loading dependencies.
#[derive(Debug)]
pub struct IrLoadError<'runtime> {
    /// Reference to the resolution error (parse or resolution failures).
    pub resolution_error: ResolutionErrorReference<'runtime>,
    /// Errors from failed Python imports encountered during load.
    pub python_import_errors: Vec<OneilError>,
}

impl<'runtime> IrLoadError<'runtime> {
    /// Creates an `IrLoadError` from a resolution error reference and a list of
    /// Python import errors.
    #[must_use]
    pub const fn new(
        resolution_error: ResolutionErrorReference<'runtime>,
        python_import_errors: Vec<OneilError>,
    ) -> Self {
        Self {
            resolution_error,
            python_import_errors,
        }
    }
}
