//! Parser error conversion for the Oneil CLI
//!
//! This module provides functionality for converting parser errors from the Oneil
//! parser library into the unified error format used by the Oneil CLI. It handles
//! syntax errors, parsing failures, and other language-related errors that occur
//! during the parsing of Oneil source files.

use std::path::Path;

use oneil_error::Error;
use oneil_parser::error::ParserError;

use crate::convert_error::file::convert as convert_file_error;

// TODO: maybe find a way to move conversions to the parser library?
//       - move the `Error` type to its own crate?

// TODO: add notes/hints for certain parser errors
//       - ExpectDecl + string starts with `~` => "Notes are only allowed at the beginning of model files and sections and after parameters and tests"
//       - ExpectDecl + string starts with `<ident> =` => "Parameters must have a label" (need to check on this one)
//       - ExpectDecl + string starts with `.*:` => "Section labels must only contain the characters [insert allowed characters]"
//       - string starts with `"` => "String literals use single quotes"
//       - string starts with `.` => "Decimal literals are not allowed to start with a `.`, try adding a leading `0`" (need to check on this one)

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
pub fn convert_all(path: &Path, parser_errors: &[ParserError]) -> Vec<Error> {
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
        let error = convert(path, file_contents.as_deref(), parser_error);
        errors.push(error);
    }

    errors
}

/// Converts a single parser error into a unified CLI error format
///
/// Takes a file path, optional file contents, and a parser error, then creates
/// a user-friendly error message with source location information if available.
///
/// # Arguments
///
/// * `path` - The path to the file that contains the parser error
/// * `file_contents` - Optional file contents for source location calculation
/// * `error` - The parser error to convert
///
/// # Returns
///
/// Returns a new `Error` instance with the parser error message and optional
/// source location information if file contents are available.
///
/// # Note
///
/// If `file_contents` is `None`, the error will be created without source
/// location information, but the error message will still be included.
pub fn convert(path: &Path, file_contents: Option<&str>, error: &ParserError) -> Error {
    let message = error.to_string();
    let location = file_contents.map(|contents| (contents, error.error_offset));

    Error::new_from_offset(path.to_path_buf(), message, location)
}
