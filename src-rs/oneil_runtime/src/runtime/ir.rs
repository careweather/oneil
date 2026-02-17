//! IR loading and resolution for the runtime.

use std::path::Path;

use indexmap::IndexSet;
use oneil_resolver as resolver;
use oneil_shared::load_result::LoadResult;

use super::Runtime;
use crate::output::{self, ast};

impl Runtime {
    /// Loads the IR for a model and all of its dependencies.
    ///
    /// # Errors
    ///
    /// Returns a
    /// [`ResolutionErrorReference`](output::reference::ResolutionErrorReference) if that
    /// model had parse or resolution errors.
    #[expect(
        clippy::missing_panics_doc,
        reason = "the panic only happens if an internal invariant is violated"
    )]
    pub fn load_ir(
        &mut self,
        path: impl AsRef<Path>,
    ) -> &LoadResult<output::ir::Model, resolver::ResolutionErrorCollection> {
        let results = resolver::load_model(&path, self);

        for (model_path, result) in results {
            let model_path = model_path.as_ref().to_path_buf();

            let (model, model_errors) = result.into_parts();

            if model_errors.is_empty() {
                self.ir_cache.insert(model_path, LoadResult::success(model));
            } else {
                self.ir_cache
                    .insert(model_path, LoadResult::partial(model, model_errors));
            }
        }

        self.ir_cache
            .get_entry(path.as_ref())
            .expect("entry was inserted in this function for the requested path")
    }
}

impl resolver::ExternalResolutionContext for Runtime {
    fn has_builtin_value(&self, identifier: &oneil_ir::Identifier) -> bool {
        self.builtins.has_builtin_value(identifier.as_str())
    }

    fn has_builtin_function(&self, identifier: &oneil_ir::Identifier) -> bool {
        self.builtins.has_builtin_function(identifier.as_str())
    }

    fn load_ast(
        &mut self,
        path: &oneil_ir::ModelPath,
    ) -> LoadResult<&ast::ModelNode, resolver::AstLoadingFailedError> {
        self.load_ast_internal(path)
            .as_ref()
            .map_err(|_e| resolver::AstLoadingFailedError)
    }

    fn load_python_import<'context>(
        &'context mut self,
        python_path: &oneil_ir::PythonPath,
    ) -> Result<IndexSet<&'context str>, resolver::PythonImportLoadingFailedError> {
        self.load_python_import(python_path.as_ref())
            .value()
            .map(|functions| functions.get_function_names().collect())
            .ok_or(resolver::PythonImportLoadingFailedError)
    }
}
