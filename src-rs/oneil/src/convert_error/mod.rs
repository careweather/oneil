//! Error conversion and formatting for the Oneil CLI
//!
//! This module provides functionality for converting various types of errors from
//! the underlying Oneil libraries into a unified error format suitable for display
//! in the CLI. It handles parser errors, file I/O errors, model loading errors,
//! and resolution errors, providing consistent error reporting across the tool.
//!
//! The module is organized into submodules:
//! - `file`: File I/O error conversion
//! - `parser`: Parser error conversion
//! - `loader`: Model loader error conversion

pub mod file;
pub mod loader;
pub mod parser;
