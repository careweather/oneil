//! Error type for parse failures.

use oneil_parser::error::ParserError;

/// Error type for parse failures.
///
/// Either a file error (e.g. source could not be read) or a list of parsing
/// errors.
#[derive(Clone, Debug)]
pub enum ParseError {
    /// The source file could not be loaded.
    FileLoadingFailed,

    /// Parsing failed with one or more errors.
    ParseErrors {
        /// Errors produced by the parser.
        errors: Vec<ParserError>,
    },
}
