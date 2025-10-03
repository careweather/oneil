//! An implementation of the `FileLoader` trait

use std::path::{Path, PathBuf};

use oneil_ast as ast;
use oneil_model_resolver::FileLoader as ModelFileLoader;
use oneil_parser as parser;
use oneil_shared::AsOneilError;

/// Type alias for parser errors with partial results
type OneilParserError =
    parser::error::ErrorsWithPartialResult<Box<ast::Model>, parser::error::ParserError>;

/// Errors that can occur during file loading operations
#[derive(Debug)]
pub enum LoadingError {
    /// Error occurred while reading the file from disk
    InvalidFile(std::io::Error),
    /// Error occurred during parsing of the file contents
    ///
    /// This indicates that the file was successfully read but contains
    /// syntax errors or other parsing issues.
    Parser(OneilParserError),
}

impl From<std::io::Error> for LoadingError {
    /// Converts an I/O error into a `LoadingError::InvalidFile`
    fn from(error: std::io::Error) -> Self {
        Self::InvalidFile(error)
    }
}

impl From<OneilParserError> for LoadingError {
    /// Converts a parser error into a `LoadingError::Parser`
    fn from(error: OneilParserError) -> Self {
        Self::Parser(error)
    }
}

/// Error indicating that a Python file does not exist
///
/// This error is used when validating Python imports in Oneil models.
/// It provides information about which Python file was expected but not found.
#[derive(Debug)]
pub struct DoesNotExistError(PathBuf);

impl DoesNotExistError {
    /// Returns the path of the file that does not exist
    ///
    /// # Returns
    ///
    /// A reference to the `PathBuf` containing the path of the missing file.
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl AsOneilError for DoesNotExistError {
    fn message(&self) -> String {
        format!("python file '{}' does not exist", self.0.display())
    }

    fn context(&self) -> Vec<oneil_shared::Context> {
        vec![]
    }
}

/// File loader implementation for Oneil source files
///
/// This struct implements the `FileLoader` trait required by the model loader system.
/// It provides functionality for parsing Oneil AST files and validating Python imports.
#[derive(Debug)]
pub struct FileLoader;

impl ModelFileLoader for FileLoader {
    type ParseError = LoadingError;
    type PythonError = DoesNotExistError;

    /// Parses a Oneil source file into an AST
    fn parse_ast(&self, path: impl AsRef<Path>) -> Result<ast::ModelNode, Self::ParseError> {
        let file_content = std::fs::read_to_string(path)?;
        let ast = parser::parse_model(&file_content, None)?;
        Ok(ast)
    }

    /// Validates that a Python import file exists
    ///
    /// Checks whether the specified Python file exists on the file system.
    /// This is used to validate Python imports referenced in Oneil models.
    ///
    /// # Note
    ///
    /// This function only checks for file existence. It does not validate
    /// the Python syntax or content of the file.
    // TODO: check if the file can be read
    fn validate_python_import(&self, path: impl AsRef<Path>) -> Result<(), Self::PythonError> {
        let path = path.as_ref();

        if path.exists() {
            Ok(())
        } else {
            Err(DoesNotExistError(path.to_path_buf()))
        }
    }
}
