//! Parser error conversion for the Oneil CLI
//!
//! This module provides functionality for converting parser errors from the Oneil
//! parser library into the unified error format used by the Oneil CLI. It handles
//! syntax errors, parsing failures, and other language-related errors that occur
//! during the parsing of Oneil source files.

use std::path::Path;

use oneil_error::OneilError;
use oneil_parser::error::ParserError;

use crate::convert_error::file::convert as convert_file_error;

/// Converts all parser errors for a file into unified CLI errors
///
/// Takes a file path and a collection of parser errors, then converts each one
/// into a unified error format. If the file cannot be read, it first adds a
/// file reading error, then processes the parser errors without source location
/// information.
///
/// # Arguments
///
/// * `path` - The path to the file that contains the parser errors
/// * `parser_errors` - A slice of parser errors to convert
///
/// # Returns
///
/// Returns a vector of `Error` instances, one for each parser error, plus
/// potentially a file reading error if the source file cannot be accessed.
///
/// # Note
///
/// This function attempts to read the source file to provide source location
/// information for the errors. If the file cannot be read, it still processes
/// the parser errors but without location information.
pub fn convert_all(path: &Path, parser_errors: &[ParserError]) -> Vec<OneilError> {
    let mut errors = Vec::new();

    let file_contents = std::fs::read_to_string(path);

    let file_contents = match file_contents {
        Ok(file_contents) => Some(file_contents),
        Err(e) => {
            // if for some reason we can't read the file, print the file reading error,
            // then print the details that we have about the error (without the file contents)
            errors.push(convert_file_error(path, &e));
            None
        }
    };

    for parser_error in parser_errors {
        let error = OneilError::from_error_with_optional_source(
            parser_error,
            path.to_path_buf(),
            file_contents.as_deref(),
        );
        errors.push(error);
    }

    errors
}
