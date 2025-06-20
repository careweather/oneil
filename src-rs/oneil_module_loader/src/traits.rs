use oneil_ast as ast;
use std::path::Path;

pub trait FileParser {
    type ParseError;
    fn parse_ast(self, path: impl AsRef<Path>) -> Result<ast::Model, Self::ParseError>;
}
