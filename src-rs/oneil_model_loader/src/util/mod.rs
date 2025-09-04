//! Utility types and traits for the Oneil model loader.
//!
//! This module provides utility types and traits that are used throughout the model
//! loading process. It includes:
//!
//! - `FileLoader` trait: Defines the interface for file parsing and validation
//! - `Stack` type: A generic stack implementation with circular dependency detection
//! - Builder types: For constructing model and parameter collections
//! - Info types: For passing information about models, submodels, and parameters
//!
//! # File Loading
//!
//! The `FileLoader` trait provides a flexible interface for parsing Oneil files and
//! validating Python imports. This allows the model loader to work with different
//! parsing implementations.
//!
//! # Circular Dependency Detection
//!
//! The `Stack` type provides circular dependency detection by tracking the loading
//! path and detecting when a model appears twice in the path.

use std::path::Path;

use oneil_ast as ast;
use oneil_ir::span::IrSpan;

pub mod builder;
pub mod builtin_ref;
pub mod context;

pub fn get_span_from_ast_span(ast_span: ast::AstSpan) -> IrSpan {
    IrSpan::new(ast_span.start(), ast_span.length())
}

/// Trait for loading and parsing Oneil model files.
///
/// This trait defines the interface that the model loader uses to parse Oneil files
/// and validate Python imports. Implementations of this trait handle the actual file
/// I/O and parsing logic, allowing the model loader to work with different parsing
/// implementations.
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
    fn parse_ast(&self, path: impl AsRef<Path>) -> Result<ast::model::ModelNode, Self::ParseError>;

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
///
/// This stack is used during model loading to track the current loading path and
/// detect circular dependencies. When a model appears twice in the loading path,
/// a circular dependency is detected.
#[derive(Debug, Clone)]
pub struct Stack<T: PartialEq + Clone> {
    items: Vec<T>,
}

impl<T: PartialEq + Clone> Stack<T> {
    /// Creates a new empty stack.
    ///
    /// # Returns
    ///
    /// A new empty `Stack`.
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    /// Pushes an item onto the top of the stack.
    ///
    /// # Arguments
    ///
    /// * `item` - The item to push onto the stack
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// Removes and returns the top item from the stack.
    ///
    /// # Returns
    ///
    /// Returns `Some(item)` if the stack is not empty, or `None` if the stack is empty.
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    /// Finds a circular dependency starting from the given item.
    ///
    /// This method searches the stack for the given item and, if found, returns
    /// a vector containing the circular dependency path. The path includes all
    /// items from the first occurrence of the item to the end of the stack,
    /// plus the item itself at the end.
    ///
    /// # Arguments
    ///
    /// * `item` - The item to search for in the stack
    ///
    /// # Returns
    ///
    /// Returns `Some(circular_dependency)` if a circular dependency is found,
    /// or `None` if the item is not in the stack.
    pub fn find_circular_dependency(&self, item: &T) -> Option<Vec<T>> {
        let item_index = self.items.iter().position(|i| i == item)?;

        let mut circular_dependency = self.items[item_index..].to_vec();
        circular_dependency.push(item.clone());

        Some(circular_dependency)
    }
}
