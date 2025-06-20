use std::path::Path;

use oneil::{
    ast, module_loader,
    parser::{self, Span},
};

pub struct FileParser;

impl<'a> module_loader::load_module::FileParser for FileParser {
    type ParseError = oneil::parser::error::ErrorsWithPartialResult<
        ast::Model,
        oneil::parser::error::ParserError<'a>,
    >;

    fn parse(self, path: impl AsRef<Path>) -> Result<ast::Model, Self::ParseError> {
        let file_content = std::fs::read_to_string(path)?;
        let input = Span::new_extra(&file_content, parser::Config::default());
        // Assuming there's a function to parse the file content into an AST
        let ast = parser::model::parse_complete(input);

        match ast {
            Ok((_rest, ast)) => Ok(ast),
            Err(nom::Err::Incomplete(needed)) => unreachable!(),
            Err(nom::Err::Error(e)) => Err(e),
            Err(nom::Err::Failure(e)) => Err(e),
        }
    }
}
