use std::{collections::HashMap, path::PathBuf};

use oneil_ast::model::ModelNode;

use crate::FileLoader;

/// A test file loader that only implements Python import validation.
///
/// This type is used for testing Python import validation logic without needing
/// to implement full AST parsing. It can be configured to validate all imports,
/// reject all imports, or selectively validate specific imports.
#[allow(
    clippy::enum_variant_names,
    reason = "'Validate' clarifies what 'All', 'None', and 'Some' refer to"
)]
pub enum TestPythonValidator {
    /// Validates all Python imports successfully.
    ValidateAll,
    /// Rejects all Python imports.
    ValidateNone,
    /// Validates only the specified Python imports.
    ValidateSome(Vec<PathBuf>),
}

impl TestPythonValidator {
    /// Creates a validator that accepts all Python imports.
    ///
    /// # Returns
    ///
    /// A `TestPythonValidator` that will return `Ok(())` for any Python import.
    pub fn validate_all() -> Self {
        Self::ValidateAll
    }

    /// Creates a validator that rejects all Python imports.
    ///
    /// # Returns
    ///
    /// A `TestPythonValidator` that will return `Err(())` for any Python import.
    pub fn validate_none() -> Self {
        Self::ValidateNone
    }

    /// Creates a validator that only accepts the specified Python imports.
    ///
    /// # Arguments
    ///
    /// * `imports` - A vector of Python file paths that should be accepted
    ///
    /// # Returns
    ///
    /// A `TestPythonValidator` that will return `Ok(())` for the specified imports
    /// and `Err(())` for all other imports.
    pub fn validate_some(imports: Vec<PathBuf>) -> Self {
        Self::ValidateSome(imports)
    }
}

impl FileLoader for TestPythonValidator {
    type ParseError = ();
    type PythonError = ();

    /// Attempts to parse an AST from a file path.
    ///
    /// This implementation always panics because `TestPythonValidator` is designed
    /// only for testing Python import validation, not AST parsing.
    ///
    /// # Arguments
    ///
    /// * `_path` - The path to the file to parse (ignored)
    ///
    /// # Panics
    ///
    /// Always panics with the message "`TestPythonLoader` does not support parsing ASTs".
    #[allow(clippy::panic_in_result_fn, reason = "this is a test implementation")]
    fn parse_ast(&self, _path: impl AsRef<std::path::Path>) -> Result<ModelNode, Self::ParseError> {
        panic!("TestPythonLoader does not support parsing ASTs");
    }

    /// Validates a Python import based on the validator's configuration.
    ///
    /// This method implements the Python import validation logic based on the
    /// validator's variant:
    ///
    /// - `ValidateAll`: Always returns `Ok(())`
    /// - `ValidateNone`: Always returns `Err(())`
    /// - `ValidateSome`: Returns `Ok(())` if the path is in the list, `Err(())` otherwise
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the Python file to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the import should be accepted, or `Err(())` if it should be rejected.
    fn validate_python_import(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), Self::PythonError> {
        let path = path.as_ref().to_path_buf();

        match self {
            Self::ValidateAll => Ok(()),
            Self::ValidateNone => Err(()),
            Self::ValidateSome(imports) => {
                if imports.contains(&path) {
                    Ok(())
                } else {
                    Err(())
                }
            }
        }
    }
}

/// A test file loader that provides predefined AST models.
///
/// This type is used for testing model loading logic with controlled AST data.
/// It maintains a map of file paths to AST models and returns the appropriate
/// model when a file is requested for parsing.
pub struct TestFileParser {
    models: HashMap<PathBuf, ModelNode>,
}

impl TestFileParser {
    /// Creates a new test file parser with the specified models.
    ///
    /// # Arguments
    ///
    /// * `models` - An iterator of (path, model) pairs that define the available AST models
    ///
    /// # Returns
    ///
    /// A new `TestFileParser` that will return the specified models when their
    /// corresponding paths are requested.
    pub fn new(models: impl IntoIterator<Item = (PathBuf, ModelNode)>) -> Self {
        Self {
            models: models.into_iter().collect(),
        }
    }

    /// Creates a new test file parser with no predefined models.
    ///
    /// This parser will return `Err(())` for any file parsing request.
    ///
    /// # Returns
    ///
    /// A new `TestFileParser` with no predefined models.
    pub fn empty() -> Self {
        Self {
            models: HashMap::new(),
        }
    }
}

impl FileLoader for TestFileParser {
    type ParseError = ();
    type PythonError = ();

    /// Attempts to parse an AST from a file path.
    ///
    /// This method looks up the requested path in the predefined models map.
    /// If the path exists, it returns the corresponding model. If the path
    /// doesn't exist, it returns `Err(())`.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to parse
    ///
    /// # Returns
    ///
    /// Returns `Ok(Model)` if the path exists in the predefined models,
    /// or `Err(())` if the path is not found.
    fn parse_ast(&self, path: impl AsRef<std::path::Path>) -> Result<ModelNode, Self::ParseError> {
        let path = path.as_ref().to_path_buf();
        self.models.get(&path).cloned().ok_or(())
    }

    /// Validates a Python import.
    ///
    /// This implementation always returns `Ok(())`, accepting all Python imports.
    /// This allows tests to focus on AST parsing behavior without worrying about
    /// Python import validation.
    ///
    /// # Arguments
    ///
    /// * `_path` - The path to the Python file to validate (ignored)
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())`.
    fn validate_python_import(
        &self,
        _path: impl AsRef<std::path::Path>,
    ) -> Result<(), Self::PythonError> {
        Ok(())
    }
}
