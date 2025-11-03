//! Utility types and traits for the Oneil model loader.

use std::{collections::HashMap, path::Path};

use oneil_ast as ast;
use oneil_ir as ir;

use crate::error::ModelImportResolutionError;

pub mod builder;
pub mod builtin_ref;
pub mod context;

pub type SubmodelMap = HashMap<ir::SubmodelName, ir::SubmodelImport>;
pub type ReferenceMap = HashMap<ir::ReferenceName, ir::ReferenceImport>;
pub type SubmodelResolutionErrors = HashMap<ir::SubmodelName, ModelImportResolutionError>;
pub type ReferenceResolutionErrors = HashMap<ir::ReferenceName, ModelImportResolutionError>;

/// Trait for loading and parsing Oneil model files.
///
/// # Associated Types
///
/// - `ParseError`: The error type returned when AST parsing fails
/// - `PythonError`: The error type returned when Python import validation fails
///
pub trait FileLoader {
    /// The error type returned when AST parsing fails.
    type ParseError: std::fmt::Debug;
    /// The error type returned when Python import validation fails.
    type PythonError: std::fmt::Debug;

    /// Parses a Oneil file into an AST.
    ///
    /// This method should read the file at the given path and parse it into a Oneil AST.
    /// The implementation is responsible for handling file I/O, syntax parsing, and
    /// any other parsing-related tasks.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the Oneil file to parse
    ///
    /// # Returns
    ///
    /// Returns `Ok(Model)` if parsing succeeds, or `Err(Self::ParseError)` if parsing fails.
    ///
    /// # Errors
    ///
    /// Returns `Err(Self::ParseError)` if the file cannot be read, parsed, or if any other
    /// parsing-related error occurs.
    fn parse_ast(&self, path: impl AsRef<Path>) -> Result<ast::ModelNode, Self::ParseError>;

    /// Validates a Python import.
    ///
    /// This method should validate that the Python model at the given path can be
    /// imported. The implementation is responsible for checking that the Python file
    /// exists, is valid Python, and can be imported successfully.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the Python file to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the import is valid, or `Err(Self::PythonError)` if validation fails.
    ///
    /// # Errors
    ///
    /// Returns `Err(Self::PythonError)` if the Python file cannot be found, is invalid,
    /// or cannot be imported successfully.
    fn validate_python_import(&self, path: impl AsRef<Path>) -> Result<(), Self::PythonError>;
}

/// A generic stack implementation with circular dependency detection.
#[derive(Debug, Clone)]
pub struct Stack<T: PartialEq + Clone> {
    items: Vec<T>,
}

impl<T: PartialEq + Clone> Stack<T> {
    /// Creates a new empty stack.
    #[must_use]
    pub const fn new() -> Self {
        Self { items: vec![] }
    }

    /// Pushes an item onto the top of the stack.
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// Removes and returns the top item from the stack.
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    /// Finds a circular dependency starting from the given item.
    ///
    /// This method searches the stack for the given item and, if found, returns
    /// a vector containing the circular dependency path. The path includes all
    /// items from the first occurrence of the item to the end of the stack,
    /// plus the item itself at the end.
    #[must_use]
    pub fn find_circular_dependency(&self, item: &T) -> Option<Vec<T>> {
        let item_index = self.items.iter().position(|i| i == item)?;

        let mut circular_dependency = self.items[item_index..].to_vec();
        circular_dependency.push(item.clone());

        Some(circular_dependency)
    }
}
