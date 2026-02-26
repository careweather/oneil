//! AST loading for the runtime.

use std::path::Path;

use oneil_parser::{self as parser, error::ParserError};
use oneil_shared::load_result::LoadResult;

use super::Runtime;
use crate::output::{ast, error::RuntimeErrors};

impl Runtime {
    /// Loads AST for a model.
    ///
    /// # Errors
    ///
    /// Returns a list of Oneil errors if the AST could not be loaded.
    pub fn load_ast(&mut self, path: impl AsRef<Path>) -> (Option<&ast::ModelNode>, RuntimeErrors) {
        let path = path.as_ref();
        self.load_ast_internal(path);

        let ast_opt = self.ast_cache.get_entry(path).and_then(LoadResult::value);
        let errors = self.get_model_errors(path);

        (ast_opt, errors)
    }

    pub(super) fn load_ast_internal(
        &mut self,
        path: impl AsRef<Path>,
    ) -> &LoadResult<ast::ModelNode, Vec<ParserError>> {
        let path = path.as_ref();
        let source_result = self.load_source_internal(path);

        let Ok(source) = source_result else {
            // if the source file could not be loaded, we return a parse error
            self.ast_cache
                .insert(path.to_path_buf(), LoadResult::failure());

            return self
                .ast_cache
                .get_entry(path)
                .expect("it was just inserted");
        };

        // parse the model and return an error if it fails
        match parser::parse_model(source, None).into_result() {
            Ok(ast) => {
                self.ast_cache
                    .insert(path.to_path_buf(), LoadResult::success(ast));

                self.ast_cache
                    .get_entry(path)
                    .expect("it was just inserted")
            }
            Err(e) => {
                // need to reload the source for lifetime reasons
                // TODO: maybe another call to `load_source` once caching works would make more sense?

                let partial_ast = e.partial_result;
                let errors = e.error_collection;

                self.ast_cache
                    .insert(path.to_path_buf(), LoadResult::partial(partial_ast, errors));

                self.ast_cache
                    .get_entry(path)
                    .expect("it was just inserted")
            }
        }
    }

    pub(super) fn parse_expression(&self, expression: &str) -> Result<ast::ExprNode, ParserError> {
        parser::parse_expression(expression, None)
    }
}
