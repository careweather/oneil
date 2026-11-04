//! AST loading for the runtime.

use std::sync::Arc;

use oneil_parser::{self as parser, error::ParserError};
use oneil_shared::{load_result::LoadResult, paths::ModelPath};

use super::Runtime;
use crate::output::{ast, error::RuntimeErrors};

impl Runtime {
    /// Loads AST for a model.
    ///
    /// # Errors
    ///
    /// Returns a list of Oneil errors if the AST could not be loaded.
    pub fn load_ast(&mut self, path: &ModelPath) -> (Option<&ast::ModelNode>, RuntimeErrors) {
        self.load_ast_internal(path);

        // doesn't matter for AST loading since we only touch one file
        let include_indirect_errors = true;

        let ast_opt = self.ast_cache.get_entry(path).and_then(LoadResult::value);
        let errors = self.get_model_diagnostics(path, include_indirect_errors);

        (ast_opt, errors)
    }

    pub(super) fn load_ast_internal(
        &mut self,
        path: &ModelPath,
    ) -> &LoadResult<ast::ModelNode, Vec<ParserError>> {
        let source_result = self.load_source_internal(&path.into());

        let Ok(source) = source_result else {
            // if the source file could not be loaded, we return a parse error
            self.ast_cache.insert(path.clone(), LoadResult::failure());

            return self
                .ast_cache
                .get_entry(path)
                .expect("it was just inserted");
        };

        // parse the model and return an error if it fails
        let rc_path: Arc<std::path::Path> = Arc::from(path.as_path());
        let rc_source: Arc<str> = Arc::from(source);
        let load_result = match parser::parse_model(
            &Arc::clone(&rc_source),
            Some(parser::Config::for_model_path(path, rc_path, rc_source)),
        )
        .into_result()
        {
            Ok(ast) => LoadResult::success(ast),
            Err(e) => {
                // need to reload the source for lifetime reasons
                // TODO: maybe another call to `load_source` once caching works would make more sense?

                let partial_ast = e.partial_result;
                let errors = e.error_collection;

                LoadResult::partial(partial_ast, errors)
            }
        };

        self.ast_cache.insert(path.clone(), load_result);

        self.ast_cache
            .get_entry(path)
            .expect("it was just inserted")
    }

    pub(super) fn parse_expression(expression: &str) -> Result<ast::ExprNode, ParserError> {
        parser::parse_expression(expression, None)
    }
}
