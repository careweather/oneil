//! Oneil Model Loader
//!
//! This crate provides functionality for loading and resolving Oneil models from files.
//! It handles the complete model loading pipeline including:
//!
//! - Parsing model files into ASTs
//! - Resolving model dependencies and imports
//! - Detecting circular dependencies
//! - Validating Python imports
//! - Building complete model collections
//!
//! # Overview
//!
//! The model loader is designed to work with any file parser that implements the `FileLoader` trait.
//! It provides a flexible interface for loading Oneil models while collecting all errors that occur
//! during the loading process.
//!
//! # Key Components
//!
//! - **Model Loading**: Main entry points for loading individual models or lists of models
//! - **Error Handling**: Comprehensive error collection and reporting
//! - **Dependency Resolution**: Handles submodel, parameter, and test resolution
//! - **Circular Dependency Detection**: Prevents infinite loading loops
//!
//! # Example
//!
//! ```ignore
//! use oneil_model_loader::{load_model, FileLoader};
//! use std::path::Path;
//!
//! struct MyFileLoader;
//!
//! impl FileLoader for MyFileLoader {
//!     type ParseError = String;
//!     type PythonError = String;
//!
//!     fn parse_ast(&self, path: impl AsRef<Path>) -> Result<oneil_ast::Model, Self::ParseError> {
//!         // Implementation here
//!         todo!()
//!     }
//!
//!     fn validate_python_import(&self, path: impl AsRef<Path>) -> Result<(), Self::PythonError> {
//!         // Implementation here
//!         todo!()
//!     }
//! }
//!
//! let file_loader = MyFileLoader;
//! let result = load_model("path/to/model.on", &file_loader);
//! ```

use std::{collections::HashSet, path::Path};

use oneil_ir as ir;

use crate::util::{Stack, builder::ModelCollectionBuilder};

pub mod error;
mod resolver;
mod util;

#[cfg(test)]
mod test;

pub use crate::error::collection::ModelErrorMap;
pub use crate::util::FileLoader;
pub use crate::util::builtin_ref::BuiltinRef;

type LoadModelOk = Box<ir::ModelCollection>;
type LoadModelErr<Ps, Py> = Box<(ir::ModelCollection, ModelErrorMap<Ps, Py>)>;

/// Loads a single model and all its dependencies.
///
/// This is the main entry point for loading a Oneil model. It loads the specified model
/// and all of its dependencies, returning either a complete `ModelCollection` or a tuple
/// containing a partial collection and any errors that occurred during loading.
///
/// # Arguments
///
/// * `model_path` - The path to the model file to load
/// * `file_parser` - The file parser implementation that handles AST parsing and Python import validation
///
/// # Returns
///
/// Returns `Ok(ModelCollection)` if the model and all its dependencies loaded successfully,
/// or `Err((ModelCollection, ModelErrorMap))` if there were errors during loading. The
/// `ModelCollection` in the error case contains all successfully loaded models.
///
/// # Errors
///
/// The function can return errors for various reasons:
///
/// - **Parse errors**: When the model file cannot be parsed into an AST
/// - **Circular dependencies**: When models have circular import dependencies
/// - **Resolution errors**: When submodels, parameters, or tests cannot be resolved
/// - **Python import errors**: When Python imports fail validation
///
/// # Example
///
/// ```ignore
/// use oneil_model_loader::{load_model, FileLoader};
/// use std::path::Path;
///
/// struct MyFileLoader;
///
/// impl FileLoader for MyFileLoader {
///     type ParseError = String;
///     type PythonError = String;
///
///     fn parse_ast(&self, path: impl AsRef<Path>) -> Result<oneil_ast::Model, Self::ParseError> {
///         // Implementation here
///         todo!()
///     }
///
///     fn validate_python_import(&self, path: impl AsRef<Path>) -> Result<(), Self::PythonError> {
///         // Implementation here
///         todo!()
///     }
/// }
///
/// let file_loader = MyFileLoader;
/// match load_model("path/to/model.on", &file_loader) {
///     Ok(collection) => println!("Successfully loaded models"),
///     Err((partial_collection, errors)) => {
///         println!("Loaded models with errors");
///         // Handle errors...
///     }
/// }
/// ```
pub fn load_model<F>(
    model_path: impl AsRef<Path>,
    builtin_ref: &impl BuiltinRef,
    file_parser: &F,
) -> Result<LoadModelOk, LoadModelErr<F::ParseError, F::PythonError>>
where
    F: FileLoader,
{
    load_model_list(&[model_path], builtin_ref, file_parser)
}

/// Loads multiple models and all their dependencies.
///
/// This function loads a list of models and all of their dependencies. It's useful when
/// you need to load multiple entry points or when models have complex interdependencies.
///
/// # Arguments
///
/// * `model_paths` - A slice of paths to model files to load
/// * `file_parser` - The file parser implementation that handles AST parsing and Python import validation
///
/// # Returns
///
/// Returns `Ok(ModelCollection)` if all models and their dependencies loaded successfully,
/// or `Err((ModelCollection, ModelErrorMap))` if there were errors during loading. The
/// `ModelCollection` in the error case contains all successfully loaded models.
///
/// # Behavior
///
/// The function processes models in the order they appear in the slice. If a model
/// has already been loaded as a dependency of an earlier model, it won't be loaded again.
/// This ensures that all dependencies are properly resolved and that circular dependencies
/// are detected.
///
/// # Errors
///
/// The function can fail with various errors during model loading:
///
/// * **Parse Errors**: If a model file cannot be parsed successfully
/// * **Python Import Errors**: If a Python import validation fails
/// * **Dependency Resolution Errors**: If model dependencies cannot be resolved
/// * **Circular Dependencies**: If circular dependencies are detected between models
/// * **Invalid Model Structure**: If a model's structure is invalid
///
/// # Example
///
/// ```ignore
///
/// use oneil_model_loader::{load_model_list, FileLoader};
/// use std::path::Path;
///
/// struct MyFileLoader;
///
/// impl FileLoader for MyFileLoader {
///     type ParseError = String;
///     type PythonError = String;
///
///     fn parse_ast(&self, path: impl AsRef<Path>) -> Result<oneil_ast::Model, Self::ParseError> {
///         // Implementation here
///         todo!()
///     }
///
///     fn validate_python_import(&self, path: impl AsRef<Path>) -> Result<(), Self::PythonError> {
///         // Implementation here
///         todo!()
///     }
/// }
///
/// let file_loader = MyFileLoader;
/// let model_paths = vec!["model1.on", "model2.on", "model3.on"];
/// match load_model_list(&model_paths, &file_loader) {
///     Ok(collection) => println!("Successfully loaded models"),
///     Err((partial_collection, errors)) => {
///         println!("Loaded models with errors");
///         // Handle errors...
///     }
/// }
/// ```
pub fn load_model_list<F>(
    model_paths: &[impl AsRef<Path>],
    builtin_ref: &impl BuiltinRef,
    file_parser: &F,
) -> Result<LoadModelOk, LoadModelErr<F::ParseError, F::PythonError>>
where
    F: FileLoader,
{
    let initial_model_paths: HashSet<_> = model_paths
        .iter()
        .map(AsRef::as_ref)
        .map(ir::ModelPath::new)
        .collect();

    let builder = ModelCollectionBuilder::new(initial_model_paths);

    let builder = model_paths.iter().fold(builder, |builder, model_path| {
        let model_path = ir::ModelPath::new(model_path.as_ref());
        let mut load_stack = Stack::new();

        resolver::load_model(
            model_path,
            builder,
            builtin_ref,
            &mut load_stack,
            file_parser,
        )
    });

    builder.try_into().map(Box::new).map_err(Box::new)
}
