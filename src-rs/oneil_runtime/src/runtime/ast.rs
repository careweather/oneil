//! AST loading for the runtime.

use std::path::Path;

use oneil_parser as parser;
use oneil_shared::load_result::LoadResult;

use super::Runtime;
use crate::output::{self, ast};

impl Runtime {
    /// Loads AST for a model.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`](output::error::ParseError) if the AST could not be loaded.
    #[expect(
        clippy::missing_panics_doc,
        reason = "the panic only happens if an internal invariant is violated"
    )]
    pub fn load_ast(
        &mut self,
        path: impl AsRef<Path>,
    ) -> &LoadResult<ast::ModelNode, output::error::ParseError> {
        let path = path.as_ref();
        let source_result = self.load_source(path);

        let source = match source_result {
            Ok(source) => source,
            Err(_error) => {
                // if the source file could not be loaded, we return a parse error
                self.ast_cache.insert(
                    path.to_path_buf(),
                    LoadResult::failure(output::error::ParseError::FileLoadingFailed),
                );

                return self
                    .ast_cache
                    .get_entry(path)
                    .expect("it was just inserted");
            }
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
                let errors = output::error::ParseError::ParseErrors {
                    errors: e.error_collection,
                };

                self.ast_cache
                    .insert(path.to_path_buf(), LoadResult::partial(partial_ast, errors));

                self.ast_cache
                    .get_entry(path)
                    .expect("it was just inserted")
            }
        }
    }
}
