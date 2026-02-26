//! IR loading and resolution for the runtime.

use std::path::Path;

use indexmap::IndexSet;
use oneil_resolver::{self as resolver, error::VariableResolutionError};
use oneil_shared::load_result::LoadResult;

use super::Runtime;
use crate::output::{self, ast, error::RuntimeErrors};

impl Runtime {
    /// Loads the IR for a model and all of its dependencies.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeErrors`] (via [`get_model_errors`](super::Runtime::get_model_errors)) if that
    /// model had parse or resolution errors.
    pub fn load_ir(
        &mut self,
        path: impl AsRef<Path>,
    ) -> (
        Option<output::reference::ModelIrReference<'_>>,
        RuntimeErrors,
    ) {
        let path = path.as_ref();
        self.load_ir_internal(path);

        let ir_opt = self
            .ir_cache
            .get_entry(path)
            .and_then(LoadResult::value)
            .map(|ir| output::reference::ModelIrReference::new(ir, &self.ir_cache));

        let errors = self.get_model_errors(path);

        (ir_opt, errors)
    }

    pub(super) fn load_ir_internal(
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

    /// Resolves an expression as if it were in the context
    /// of the given model.
    pub(super) fn resolve_expr_in_model(
        &mut self,
        expr_ast: &ast::ExprNode,
        file: &Path,
    ) -> Result<output::ir::Expr, Vec<VariableResolutionError>> {
        resolver::resolve_expr_in_model(expr_ast, file, self)
    }
}

impl resolver::ExternalResolutionContext for Runtime {
    fn has_builtin_value(&self, identifier: &oneil_ir::Identifier) -> bool {
        self.builtins.has_builtin_value(identifier.as_str())
    }

    fn has_builtin_function(&self, identifier: &oneil_ir::Identifier) -> bool {
        self.builtins.has_builtin_function(identifier.as_str())
    }

    fn has_builtin_unit(&self, name: &str) -> bool {
        self.builtins.get_unit(name).is_some()
    }

    fn available_prefixes(&self) -> impl Iterator<Item = (&str, f64)> {
        self.builtins.builtin_prefixes()
    }

    fn unit_supports_si_prefixes(&self, name: &str) -> bool {
        self.builtins.unit_supports_si_prefixes(name)
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
        self.load_python_import_internal(python_path.as_ref())
            .as_ref()
            .ok()
            .map(|functions| functions.get_function_names().collect())
            .ok_or(resolver::PythonImportLoadingFailedError)
    }
}
