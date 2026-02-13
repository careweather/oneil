//! IR loading and resolution for the runtime.

use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};
use oneil_resolver as resolver;
use oneil_shared::{error::OneilError, load_result::LoadResult};

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
    pub fn load_ir(&mut self, path: impl AsRef<Path>) -> output::IrLoadResult<'_> {
        let results = resolver::load_model(&path, self);

        for (model_path, result) in results {
            let model_path = model_path.as_ref().to_path_buf();

            let (model, model_errors) = result.into_parts();

            // If the AST failed to load during resolution, we insert
            // the parse error that caused it to fail
            if !ast_loaded {
                let parse_err = self
                    .ast_cache
                    .get_errors(&model_path)
                    .expect("should have ast error");

                self.ir_cache.insert_err(
                    model_path.clone(),
                    output::error::ResolutionError::Parse(parse_err.clone()),
                );
                continue;
            }

            self.process_model_result(model_path, model, model_errors);
        }

        let python_import_errors = failed_python_imports
            .into_iter()
            .map(|python_path| {
                self.python_import_cache
                    .get_error(python_path.as_ref())
                    .expect("should have error")
                    .clone()
            })
            .collect::<Vec<OneilError>>();

        let entry = self
            .ir_cache
            .get_entry(path.as_ref())
            .expect("entry was inserted in this function for the requested path");

        match entry.as_ref() {
            Ok(model) => {
                assert!(python_import_errors.is_empty());
                output::ir_result::IrLoadResult::ok(output::reference::ModelIrReference::new(
                    model,
                    &self.ir_cache,
                ))
            }

            Err(resolution_error) => output::ir_result::IrLoadResult::err(
                output::reference::ResolutionErrorReference::new(resolution_error, &self.ir_cache),
                python_import_errors,
            ),
        }
    }

    fn process_model_result(
        &mut self,
        model_path: PathBuf,
        model: oneil_ir::Model,
        model_errors: oneil_resolver::ResolutionErrorCollection,
    ) {
        let source = self
            .source_cache
            .get(&model_path)
            .expect("it has already been loaded previously");

        let (
            circular_dependency_errors,
            python_import_errors,
            model_import_errors,
            parameter_errors,
            test_errors,
        ) = model_errors.into_parts();

        let circular_dependency_oneil: Vec<OneilError> = circular_dependency_errors
            .into_iter()
            .map(|e| OneilError::from_error_with_source(&e, model_path.clone(), source))
            .collect();

        let python_import_oneil: Vec<OneilError> = python_import_errors
            .into_values()
            .map(|e| OneilError::from_error_with_source(&e, model_path.clone(), source))
            .collect();

        let model_import_oneil: Vec<OneilError> = model_import_errors
            .into_values()
            .map(|(_, e)| OneilError::from_error_with_source(&e, model_path.clone(), source))
            .collect();

        let parameter_errors_oneil: IndexMap<String, Vec<OneilError>> = parameter_errors
            .into_iter()
            .map(|(name, errs)| {
                (
                    name.as_str().to_string(),
                    errs.into_iter()
                        .map(|e| OneilError::from_error_with_source(&e, model_path.clone(), source))
                        .collect(),
                )
            })
            .collect();

        let test_errors_oneil: Vec<OneilError> = test_errors
            .into_iter()
            .flat_map(|(_test_index, errors)| {
                errors
                    .into_iter()
                    .map(|e| OneilError::from_error_with_source(&e, model_path.clone(), source))
                    .collect::<Vec<_>>()
            })
            .collect();

        let has_errors = !circular_dependency_oneil.is_empty()
            || !python_import_oneil.is_empty()
            || !model_import_oneil.is_empty()
            || !parameter_errors_oneil.is_empty()
            || !test_errors_oneil.is_empty();

        if has_errors {
            let python_map = IndexMap::from_iter([(model_path.clone(), python_import_oneil)]);
            let model_import_map = IndexMap::from_iter([(model_path.clone(), model_import_oneil)]);

            self.ir_cache.insert_err(
                model_path,
                output::error::ResolutionError::ResolutionErrors {
                    partial_ir: Box::new(model),
                    circular_dependency_errors: circular_dependency_oneil,
                    python_import_errors: python_map,
                    model_import_errors: model_import_map,
                    parameter_errors: parameter_errors_oneil,
                    test_errors: test_errors_oneil,
                },
            );
        } else {
            self.ir_cache.insert_ok(model_path, model);
        }
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
        self.load_ast(path)
            .as_ref()
            .map_err(|_e| resolver::AstLoadingFailedError)
    }

    fn load_python_import(
        &mut self,
        python_path: &oneil_ir::PythonPath,
    ) -> Result<IndexSet<String>, resolver::PythonImportLoadingFailedError> {
        self.load_python_import(python_path.as_ref())
            .map_err(|_error| resolver::PythonImportLoadingFailedError)
    }
}
