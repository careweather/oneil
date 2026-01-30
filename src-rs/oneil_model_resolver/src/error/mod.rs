//! Error handling for the Oneil model loader.

pub mod circular_dependency;
pub mod resolution;
pub mod util;

pub use circular_dependency::CircularDependencyError;
pub use resolution::{
    ModelImportResolutionError, ParameterResolutionError, PythonImportResolutionError,
    ResolutionErrors, VariableResolutionError,
};
pub use util::{combine_error_list, combine_errors, convert_errors, split_ok_and_errors};
