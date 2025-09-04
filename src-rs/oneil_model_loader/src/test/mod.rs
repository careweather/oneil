//! Test utilities for the model loader.
//!
//! This module provides test utilities that implement the `FileLoader` trait for
//! testing purposes. These utilities allow tests to control the behavior of
//! file parsing and Python import validation without requiring actual files.
//!
//! # Test Types
//!
//! - `TestPythonValidator`: A file loader that only implements Python import validation
//! - `TestFileParser`: A file loader that provides predefined AST models for testing
//!
//! # Usage
//!
//! These test utilities are primarily used in the model's own tests to verify
//! the behavior of the model loading system under various conditions, such as
//! successful imports, failed imports, and different AST structures.

mod builtin_ref;
mod context;
mod file_loader;
mod helper;

pub use builtin_ref::TestBuiltinRef;
pub use context::TestContext;
pub use file_loader::{TestFileParser, TestPythonValidator};
