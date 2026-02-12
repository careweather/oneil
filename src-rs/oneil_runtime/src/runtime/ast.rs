//! AST loading for the runtime.

use std::path::Path;

use oneil_parser as parser;
use oneil_shared::error::OneilError;

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
    ) -> Result<&ast::Model, output::error::ParseError> {
        let path = path.as_ref();
        let source = self
            .load_source(path)
            .map_err(output::error::ParseError::File)?;

        // parse the model and return an error if it fails
        match parser::parse_model(source, None).into_result() {
            Ok(ast) => {
                self.ast_cache
                    .insert_ok(path.to_path_buf(), ast.take_value());
                let ast = self.ast_cache.get(path).expect("it was just inserted");

                Ok(ast)
            }
            Err(e) => {
                // need to reload the source for lifetime reasons
                // TODO: maybe another call to `load_source` once caching works would make more sense?
                let source = self
                    .source_cache
                    .get(path)
                    .expect("it has already been loaded previously");
                let errors = e
                    .error_collection
                    .into_iter()
                    .map(|err| OneilError::from_error_with_source(&err, path.to_path_buf(), source))
                    .collect::<Vec<OneilError>>();

                let partial_ast = e.partial_result.take_value();
                let partial_ast_for_error = partial_ast.clone();
                self.ast_cache
                    .insert_err(path.to_path_buf(), partial_ast, errors.clone());

                Err(output::error::ParseError::ParseErrors {
                    errors,
                    partial_ast: Box::new(partial_ast_for_error),
                })
            }
        }
    }
}
