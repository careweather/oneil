//! Oneil Module Loader
//!
//! This crate provides functionality for loading and resolving Oneil modules from files.
//! It handles the complete module loading pipeline including:
//!
//! - Parsing module files into ASTs
//! - Resolving module dependencies and imports
//! - Detecting circular dependencies
//! - Validating Python imports
//! - Building complete module collections
//!
//! # Overview
//!
//! The module loader is designed to work with any file parser that implements the `FileLoader` trait.
//! It provides a flexible interface for loading Oneil modules while collecting all errors that occur
//! during the loading process.
//!
//! # Key Components
//!
//! - **Module Loading**: Main entry points for loading individual modules or lists of modules
//! - **Error Handling**: Comprehensive error collection and reporting
//! - **Dependency Resolution**: Handles submodel, parameter, and test resolution
//! - **Circular Dependency Detection**: Prevents infinite loading loops
//!
//! # Example
//!
//! ```ignore
//! use oneil_model_loader::{load_module, FileLoader};
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
//! let result = load_module("path/to/module.on", &file_loader);
//! ```

#![warn(missing_docs)]

use std::{collections::HashSet, path::Path};

use oneil_ir::{module::ModuleCollection, reference::ModulePath};

use crate::{
    error::collection::ModuleErrorMap,
    util::{Stack, builder::ModuleCollectionBuilder},
};

mod error;
mod loader;
mod util;

#[cfg(test)]
mod test;

pub use crate::util::FileLoader;

/// Loads a single module and all its dependencies.
///
/// This is the main entry point for loading a Oneil module. It loads the specified module
/// and all of its dependencies, returning either a complete `ModuleCollection` or a tuple
/// containing a partial collection and any errors that occurred during loading.
///
/// # Arguments
///
/// * `module_path` - The path to the module file to load
/// * `file_parser` - The file parser implementation that handles AST parsing and Python import validation
///
/// # Returns
///
/// Returns `Ok(ModuleCollection)` if the module and all its dependencies loaded successfully,
/// or `Err((ModuleCollection, ModuleErrorMap))` if there were errors during loading. The
/// `ModuleCollection` in the error case contains all successfully loaded modules.
///
/// # Errors
///
/// The function can return errors for various reasons:
///
/// - **Parse errors**: When the module file cannot be parsed into an AST
/// - **Circular dependencies**: When modules have circular import dependencies
/// - **Resolution errors**: When submodels, parameters, or tests cannot be resolved
/// - **Python import errors**: When Python imports fail validation
///
/// # Example
///
/// ```ignore
/// use oneil_model_loader::{load_module, FileLoader};
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
/// match load_module("path/to/module.on", &file_loader) {
///     Ok(collection) => println!("Successfully loaded modules"),
///     Err((partial_collection, errors)) => {
///         println!("Loaded modules with errors");
///         // Handle errors...
///     }
/// }
/// ```
pub fn load_module<F>(
    module_path: impl AsRef<Path>,
    file_parser: &F,
) -> Result<
    ModuleCollection,
    (
        ModuleCollection,
        ModuleErrorMap<F::ParseError, F::PythonError>,
    ),
>
where
    F: FileLoader,
{
    load_module_list(&[module_path], file_parser)
}

/// Loads multiple modules and all their dependencies.
///
/// This function loads a list of modules and all of their dependencies. It's useful when
/// you need to load multiple entry points or when modules have complex interdependencies.
///
/// # Arguments
///
/// * `module_paths` - A slice of paths to module files to load
/// * `file_parser` - The file parser implementation that handles AST parsing and Python import validation
///
/// # Returns
///
/// Returns `Ok(ModuleCollection)` if all modules and their dependencies loaded successfully,
/// or `Err((ModuleCollection, ModuleErrorMap))` if there were errors during loading. The
/// `ModuleCollection` in the error case contains all successfully loaded modules.
///
/// # Behavior
///
/// The function processes modules in the order they appear in the slice. If a module
/// has already been loaded as a dependency of an earlier module, it won't be loaded again.
/// This ensures that all dependencies are properly resolved and that circular dependencies
/// are detected.
///
/// # Example
///
/// ```ignore
///
/// use oneil_model_loader::{load_module_list, FileLoader};
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
/// let module_paths = vec!["module1.on", "module2.on", "module3.on"];
/// match load_module_list(&module_paths, &file_loader) {
///     Ok(collection) => println!("Successfully loaded modules"),
///     Err((partial_collection, errors)) => {
///         println!("Loaded modules with errors");
///         // Handle errors...
///     }
/// }
/// ```
pub fn load_module_list<F>(
    module_paths: &[impl AsRef<Path>],
    file_parser: &F,
) -> Result<
    ModuleCollection,
    (
        ModuleCollection,
        ModuleErrorMap<F::ParseError, F::PythonError>,
    ),
>
where
    F: FileLoader,
{
    let initial_module_paths: HashSet<_> = module_paths
        .iter()
        .map(|p| ModulePath::new(p.as_ref().to_path_buf()))
        .collect();

    let builder = ModuleCollectionBuilder::new(initial_module_paths);

    let builder = module_paths.iter().fold(builder, |builder, module_path| {
        let module_path = ModulePath::new(module_path.as_ref().to_path_buf());
        let mut load_stack = Stack::new();

        loader::load_module(module_path, builder, &mut load_stack, file_parser)
    });

    builder.try_into()
}
