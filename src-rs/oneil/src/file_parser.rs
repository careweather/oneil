use std::path::Path;

use oneil_ast as ast;
use oneil_parser as parser;
use oneil_parser::error as parser_error;

type OneilParserError =
    parser_error::ErrorsWithPartialResult<ast::Model, parser_error::ParserError>;

#[derive(Debug)]
pub enum LoadingError {
    InvalidFile(std::io::Error),
    Parser(OneilParserError),
}

impl From<std::io::Error> for LoadingError {
    fn from(error: std::io::Error) -> Self {
        LoadingError::InvalidFile(error)
    }
}

impl From<OneilParserError> for LoadingError {
    fn from(error: OneilParserError) -> Self {
        LoadingError::Parser(error)
    }
}

#[derive(Debug)]
pub struct FileLoader;

impl<'a> oneil_module_loader::FileLoader for FileLoader {
    type ParseError = LoadingError;

    fn parse_ast(&self, path: impl AsRef<Path>) -> Result<ast::Model, Self::ParseError> {
        let file_content = std::fs::read_to_string(path)?;
        let ast = parser::parse_model(&file_content, None)?;
        Ok(ast)
    }

    fn file_exists(&self, path: impl AsRef<Path>) -> bool {
        path.as_ref().exists()
    }
}
