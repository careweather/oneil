use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};
use oneil_ast as ast;
use oneil_eval::value::{Unit, Value};
use oneil_eval::{self as eval, EvalError};
use oneil_ir as ir;
use oneil_model_resolver as model_resolver;
use oneil_parser as parser;
use oneil_shared::error::OneilError;
use oneil_shared::span::Span;

use crate::cache::{AstCache, IrCache, SourceCache};
use crate::{error::FileError, std_builtin::StdBuiltins};

/// Runtime for the Oneil programming language.
///
/// The runtime manages caches for source files, ASTs, and IR, and provides
/// methods to load and process Oneil models.
#[derive(Debug)]
pub struct Runtime {
    source_cache: SourceCache,
    ast_cache: AstCache,
    ir_cache: IrCache,
    builtins: StdBuiltins,
}

impl Runtime {
    /// Creates a new runtime instance with empty caches.
    pub fn new() -> Self {
        Self {
            source_cache: SourceCache::new(),
            ast_cache: AstCache::new(),
            ir_cache: IrCache::new(),
            builtins: StdBuiltins::new(),
        }
    }

    /// Evaluates a model and returns the result.
    pub fn eval_model(&mut self, path: impl AsRef<Path>) -> Result<(), Vec<OneilError>> {
        let model_map = eval::eval_model(path, self);
        todo!()
    }

    /// Loads the IR for a model and all of its dependencies.
    pub fn load_ir_for_model_and_dependencies(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<IrModelMap<'_>, PartialResultWithErrors<IrModelMap<'_>>> {
        let results = model_resolver::load_model(&path, self);

        let mut models = IndexSet::new();
        let mut errors = Vec::new();

        for (model_path, result) in results {
            let model_path = model_path.as_ref().to_path_buf();

            let (model, model_errors, circular_dependency_errors, ast_loaded) = result.into_parts();

            // push the model IR to the cache
            self.ir_cache.insert(model_path.clone(), model);

            models.insert(model_path.clone());

            // if the AST for the model has not been loaded,
            // return the source and AST errors
            if !ast_loaded {
                let source_error = self.source_cache.get_error(&model_path).cloned();
                let ast_errors = self.ast_cache.get_errors(&model_path).map(Vec::from);

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
                self.ir_cache
                    .insert_errors(model_path.clone(), model_errors_as_oneil.clone());
                errors.extend(model_errors_as_oneil);
            }
        }

        let models = models
            .into_iter()
            .map(|model_path| {
                let model = self
                    .ir_cache
                    .get(&model_path)
                    .expect("it has already been loaded previously");
                (model_path, model)
            })
            .collect::<IndexMap<PathBuf, &ir::Model>>();

        if errors.is_empty() {
            Ok(models)
        } else {
            Err(PartialResultWithErrors {
                result: models,
                errors,
            })
        }
    }

    /// Loads AST for a model.
    ///
    /// On success, returns the AST. On failure, returns the errors
    /// and a partial AST if available.
    pub fn load_ast(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<&ast::Model, PartialResultWithErrors<Option<&ast::Model>>> {
        let path = path.as_ref();
        let source = self
            .load_source(path)
            .map_err(|e| PartialResultWithErrors {
                result: None,
                errors: vec![*e],
            })?;

        // parse the model and return an error if it fails
        match parser::parse_model(source, None) {
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
                    .errors
                    .into_iter()
                    .map(|e| OneilError::from_error_with_source(&e, path.to_path_buf(), source))
                    .collect::<Vec<OneilError>>();

                let partial_ast = e.partial_result;
                self.ast_cache
                    .insert_err(path.to_path_buf(), *partial_ast, errors.clone());

                let ast = self
                    .ast_cache
                    .get(path)
                    .expect("it has already been loaded previously");

                Err(PartialResultWithErrors {
                    result: Some(ast),
                    errors,
                })
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
                self.source_cache.insert_ok(path.to_path_buf(), source);

                let source = self.source_cache.get(path).expect("it was just inserted");

                Ok(source)
            }
            Err(e) => {
                let error = FileError::new(path, e);
                let error = OneilError::from_error(&error, path.to_path_buf());

                self.source_cache
                    .insert_err(path.to_path_buf(), error.clone());

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
        self.builtins.has_builtin_value(identifier.as_str())
    }

    fn has_builtin_function(&self, identifier: &ir::Identifier) -> bool {
        self.builtins.has_builtin_function(identifier.as_str())
    }

    fn load_ast(
        &mut self,
        path: &ir::ModelPath,
    ) -> Result<&ast::Model, model_resolver::AstLoadingFailedError> {
        self.load_ast(path)
            .map_err(|_e| model_resolver::AstLoadingFailedError)
    }

    fn load_python_import(
        &mut self,
        python_path: &ir::PythonPath,
    ) -> Result<(), model_resolver::PythonImportLoadingFailedError> {
        todo!()
    }
}

impl eval::ExternalEvaluationContext for Runtime {
    fn lookup_ir(&self, path: impl AsRef<Path>) -> Option<&ir::Model> {
        self.ir_cache.get(path.as_ref())
    }

    fn lookup_builtin_variable(&self, identifier: &ir::Identifier) -> Option<&Value> {
        self.builtins.get_value(identifier.as_str())
    }

    fn evaluate_builtin_function(
        &self,
        identifier: &ir::Identifier,
        identifier_span: Span,
        args: Vec<(Value, Span)>,
    ) -> Option<Result<Value, Vec<EvalError>>> {
        let function = self.builtins.get_function(identifier.as_str())?;
        Some(function(identifier_span, args))
    }

    fn lookup_unit(&self, name: &str) -> Option<&Unit> {
        self.builtins.get_unit(name)
    }

    fn available_prefixes(&self) -> impl Iterator<Item = (&str, f64)> {
        self.builtins.builtin_prefixes()
    }
}

type IrModelMap<'runtime> = IndexMap<PathBuf, &'runtime ir::Model>;

pub struct PartialResultWithErrors<T> {
    pub result: T,
    pub errors: Vec<OneilError>,
}
