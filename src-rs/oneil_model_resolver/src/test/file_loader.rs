use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use oneil_ast as ast;

use crate::FileLoader;

/// A test file loader that only implements Python import validation.
///
/// This type is used for testing Python import validation logic without needing
/// to implement full AST parsing. It can be configured to validate all imports,
/// reject all imports, or selectively validate specific imports.
#[expect(
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
    pub fn validate_all() -> Self {
        Self::ValidateAll
    }

    /// Creates a validator that rejects all Python imports.
    pub fn validate_none() -> Self {
        Self::ValidateNone
    }

    /// Creates a validator that only accepts the specified Python imports.
    pub fn validate_some(imports: impl IntoIterator<Item = impl AsRef<Path>>) -> Self {
        let imports = imports
            .into_iter()
            .map(|path| path.as_ref().to_path_buf())
            .collect();
        Self::ValidateSome(imports)
    }
}

impl FileLoader for TestPythonValidator {
    type ParseError = ();
    type PythonError = ();

    /// Attempts to parse an AST from a file path.
    #[expect(clippy::panic_in_result_fn, reason = "this is a test implementation")]
    fn parse_ast(&self, _path: impl AsRef<Path>) -> Result<ast::ModelNode, Self::ParseError> {
        panic!("TestPythonLoader does not support parsing ASTs");
    }

    /// Validates a Python import based on the validator's configuration.
    fn validate_python_import(&self, path: impl AsRef<Path>) -> Result<(), Self::PythonError> {
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
    models: HashMap<PathBuf, ast::ModelNode>,
}

impl TestFileParser {
    /// Creates a new test file parser with the specified models.
    pub fn new(models: impl IntoIterator<Item = (impl AsRef<Path>, ast::ModelNode)>) -> Self {
        let models = models
            .into_iter()
            .map(|(path, model)| (path.as_ref().to_path_buf(), model))
            .collect();

        Self { models }
    }

    /// Creates a new test file parser with no predefined models.
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
    fn parse_ast(&self, path: impl AsRef<Path>) -> Result<ast::ModelNode, Self::ParseError> {
        let path = path.as_ref().to_path_buf();
        self.models.get(&path).cloned().ok_or(())
    }

    /// Validates a Python import.
    fn validate_python_import(
        &self,
        _path: impl AsRef<std::path::Path>,
    ) -> Result<(), Self::PythonError> {
        Ok(())
    }
}
