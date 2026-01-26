use std::path::Path;

use oneil_parser::{self as parser, Config};
use oneil_shared::error::OneilError;

use crate::{
    cache::{AstCache, SourceCache},
    debug::{ast, ir},
    error::FileError,
};

pub struct Runtime {
    source_cache: SourceCache,
    ast_cache: AstCache,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            source_cache: SourceCache::new(),
            ast_cache: AstCache::new(),
        }
    }

    pub fn debug_load_ast(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<&ast::ModelNode, Vec<OneilError>> {
        self.load_ast(path)
    }

    fn load_ast(&mut self, path: impl AsRef<Path>) -> Result<&ast::ModelNode, Vec<OneilError>> {
        let content = self.load_source(&path).map_err(|e| vec![*e])?;
        let parse_result = parser::parse_model(content, Some(Config::default()));

        match parse_result {
            Ok(ast) => {
                let ast = self.ast_cache.insert_ast(path.as_ref().to_path_buf(), ast);

                Ok(ast)
            }
            Err(e) => {
                let errors = e
                    .errors
                    .into_iter()
                    .map(|e| OneilError::from_error(&e, path.as_ref().to_path_buf()))
                    .collect();

                let errors = self
                    .ast_cache
                    .insert_errors(path.as_ref().to_path_buf(), errors);

                let errors = errors.to_vec();

                Err(errors)
            }
        }
    }

    fn load_source(&mut self, path: impl AsRef<Path>) -> Result<&str, Box<OneilError>> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path);

        match content {
            Ok(content) => {
                let content = self.source_cache.insert_source(path.to_path_buf(), content);
                Ok(content)
            }
            Err(error) => {
                let error = FileError::new(path, &error);
                let error = OneilError::from_error(&error, path.to_path_buf());
                let error = self
                    .source_cache
                    .insert_error(path.to_path_buf(), error.clone());

                Err(Box::new(error))
            }
        }
    }
}
