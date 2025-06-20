use std::path::Path;

use crate::ast;
use crate::module_loader::{Module, ModuleCollection, error::ModuleLoaderError};

pub trait FileParser {
    type ParseError;
    fn parse(self, path: impl AsRef<Path>) -> Result<ast::Model, Self::ParseError>;
}

pub fn load_module<F>(
    path: impl AsRef<Path>,
    file_parser: F,
) -> Result<ModuleCollection, ModuleLoaderError<F::ParseError>>
where
    F: FileParser,
{
    let file_ast = file_parser.parse(path)?;

    let result = traverse_ast::<F::ParseError>(file_ast);

    todo!()
}

fn traverse_ast<E>(ast: ast::Model) -> Result<Module, ModuleLoaderError<E>> {
    todo!()
}
