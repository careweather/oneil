use oneil_ast as ast;
use std::path::Path;

// TODO: rename to FileLoader?
pub trait FileParser {
    type ParseError;
    fn parse_ast(&self, path: impl AsRef<Path>) -> Result<ast::Model, Self::ParseError>;
    fn file_exists(&self, path: impl AsRef<Path>) -> bool;
}
