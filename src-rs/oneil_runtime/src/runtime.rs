use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_ast as ast;
use oneil_ir as ir;
use oneil_model_resolver as model_resolver;
use oneil_parser as parser;
use oneil_shared::error::OneilError;

use crate::error::FileError;

/// Runtime for the Oneil programming language.
///
/// The runtime manages caches for source files, ASTs, and IR, and provides
/// methods to load and process Oneil models.
#[derive(Debug)]
pub struct Runtime {
    source_cache: IndexMap<PathBuf, String>,
    source_errors: IndexMap<PathBuf, OneilError>,
    ast_cache: IndexMap<PathBuf, ast::Model>,
    ast_errors: IndexMap<PathBuf, Vec<OneilError>>,
    ir_cache: IndexMap<PathBuf, ir::Model>,
    ir_errors: IndexMap<PathBuf, Vec<OneilError>>,
}

impl Runtime {
    /// Creates a new runtime instance with empty caches.
    pub fn new() -> Self {
        Self {
            source_cache: IndexMap::new(),
            source_errors: IndexMap::new(),
            ast_cache: IndexMap::new(),
            ast_errors: IndexMap::new(),
            ir_cache: IndexMap::new(),
            ir_errors: IndexMap::new(),
        }
    }

    /// Loads IR for a model.
    pub fn load_ir(&mut self, path: impl AsRef<Path>) -> Result<&ir::Model, Vec<OneilError>> {
        let results = model_resolver::load_model(&path, self);

        let mut errors = Vec::new();
        for (model_path, result) in results {
            let model_path = model_path.as_ref().to_path_buf();

            let (model, model_errors, circular_dependency_errors, ast_loaded) = result.into_parts();

            // push the model IR to the cache
            self.ir_cache.insert(model_path.clone(), model);

            // if the AST for the model has not been loaded,
            // return the source and AST errors
            if !ast_loaded {
                let source_error = self.source_errors.get(&model_path).cloned();
                let ast_errors = self.ast_errors.get(&model_path).cloned();

                errors.extend(source_error.iter().cloned());
                errors.extend(ast_errors.iter().flatten().cloned());

                continue;
            }

            // otherwise, get the model resolution and circular dependency errors, if any
            let mut model_errors_as_oneil = Vec::new();

            let source = self
                .source_cache
                .get(&model_path)
                .expect("it has already been loaded previously");

            let (python_import_errors, model_import_errors, parameter_errors, test_errors) =
                model_errors.into_parts();

            let import_errors = python_import_errors
                .into_values()
                .map(|e| OneilError::from_error_with_source(&e, model_path.clone(), source));
            model_errors_as_oneil.extend(import_errors);

            let model_import_errors = model_import_errors
                .into_values()
                .map(|(_, e)| OneilError::from_error_with_source(&e, model_path.clone(), source));
            model_errors_as_oneil.extend(model_import_errors);

            let parameter_errors = parameter_errors
                .into_values()
                .flatten()
                .map(|e| OneilError::from_error_with_source(&e, model_path.clone(), source));
            model_errors_as_oneil.extend(parameter_errors);

            let test_errors = test_errors
                .into_values()
                .flatten()
                .map(|e| OneilError::from_error_with_source(&e, model_path.clone(), source));
            model_errors_as_oneil.extend(test_errors);

            let circular_dependency_errors = circular_dependency_errors
                .into_iter()
                .map(|e| OneilError::from_error_with_source(&e, model_path.clone(), source));
            model_errors_as_oneil.extend(circular_dependency_errors);

            // if there are any model errors,
            // add them to the cache and to the overall error collection
            if !model_errors_as_oneil.is_empty() {
                self.ir_errors
                    .insert(model_path.clone(), model_errors_as_oneil.clone());
                errors.extend(model_errors_as_oneil);
            }
        }

        if errors.is_empty() {
            let ir_model = self
                .ir_cache
                .get(&path.as_ref().to_path_buf())
                .expect("it has already been loaded previously");

            Ok(ir_model)
        } else {
            Err(errors)
        }
    }

    /// Loads AST for a model.
    pub fn load_ast(&mut self, path: impl AsRef<Path>) -> Result<&ast::Model, Vec<OneilError>> {
        let path = path.as_ref();
        let source = self.load_source(path).map_err(|e| vec![*e])?;

        // parse the model and return an error if it fails
        match parser::parse_model(source, None) {
            Ok(ast) => {
                self.ast_cache.insert(path.to_path_buf(), ast.take_value());
                let ast = self.ast_cache.get(path).expect("it was just inserted");

                Ok(ast)
            }
            Err(e) => {
                let ast = e.partial_result;
                self.ast_cache.insert(path.to_path_buf(), *ast);

                // need to reload the source for lifetime reasons
                // TODO: maybe another call to `load_source` once caching works would make more sense?
                let source = self
                    .source_cache
                    .get(path)
                    .expect("it has already been loaded previously");

                let errors = e
                    .errors
                    .into_iter()
                    .map(|e| OneilError::from_error_with_source(&e, path.to_path_buf(), source))
                    .collect::<Vec<OneilError>>();
                self.ast_errors.insert(path.to_path_buf(), errors.clone());

                Err(errors)
            }
        }
    }

    /// Loads source code from a file.
    pub fn load_source(&mut self, path: impl AsRef<Path>) -> Result<&str, Box<OneilError>> {
        let path = path.as_ref();

        // Read the source code from the file
        match std::fs::read_to_string(path) {
            Ok(source) => {
                // If it's found, insert it into the loaded models map
                // and return it
                self.source_cache.insert(path.to_path_buf(), source);

                let source = self.source_cache.get(path).expect("it was just inserted");

                Ok(source)
            }
            Err(e) => {
                let error = FileError::new(path, e);
                let error = OneilError::from_error(&error, path.to_path_buf());

                self.source_errors.insert(path.to_path_buf(), error.clone());

                Err(Box::new(error))
            }
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl model_resolver::ExternalResolutionContext for Runtime {
    fn has_builtin_value(&self, identifier: &ir::Identifier) -> bool {
        todo!()
    }

    fn has_builtin_function(&self, identifier: &ir::Identifier) -> bool {
        todo!()
    }

    fn load_ast(
        &mut self,
        path: &ir::ModelPath,
    ) -> Result<&ast::ModelNode, model_resolver::AstLoadingFailedError> {
        todo!()
    }

    fn load_python_import(
        &mut self,
        python_path: &ir::PythonPath,
    ) -> Result<(), model_resolver::PythonImportLoadingFailedError> {
        todo!()
    }
}
