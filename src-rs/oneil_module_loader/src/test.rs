//! Test utilities for the module loader

use std::path::PathBuf;

use crate::FileLoader;

pub enum TestPythonValidator {
    ValidateAll,
    ValidateNone,
    ValidateSome(Vec<PathBuf>),
}

impl TestPythonValidator {
    pub fn validate_all() -> Self {
        Self::ValidateAll
    }

    pub fn validate_none() -> Self {
        Self::ValidateNone
    }

    pub fn validate_some(imports: Vec<PathBuf>) -> Self {
        Self::ValidateSome(imports)
    }
}

impl FileLoader for TestPythonValidator {
    type ParseError = ();
    type PythonError = ();

    fn parse_ast(
        &self,
        _path: impl AsRef<std::path::Path>,
    ) -> Result<oneil_ast::Model, Self::ParseError> {
        panic!("TestPythonLoader does not support parsing ASTs");
    }

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

pub struct TestModelParser;

impl FileLoader for TestModelParser {
    type ParseError = ();
    type PythonError = ();

    fn parse_ast(
        &self,
        _path: impl AsRef<std::path::Path>,
    ) -> Result<oneil_ast::Model, Self::ParseError> {
        todo!()
    }

    fn validate_python_import(
        &self,
        _path: impl AsRef<std::path::Path>,
    ) -> Result<(), Self::PythonError> {
        panic!("TestModelParser does not support validating python imports");
    }
}
