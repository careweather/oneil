use std::path::Path;

use indexmap::IndexMap;
use oneil_ast as ast;
use oneil_ir as ir;
use oneil_model_resolver::{self as model_resolver, ModelResolutionResult};
use oneil_parser::{self as parser, Config};
use oneil_shared::error::OneilError;

use crate::{
    cache::{AstCache, IrCache, SourceCache},
    debug,
    error::FileError,
    std_builtin::StdBuiltins,
};

/// Runtime for the Oneil programming language.
///
/// The runtime manages caches for source files, ASTs, and IR, and provides
/// methods to load and process Oneil models.
pub struct Runtime {
    builtins: StdBuiltins,
    source_cache: SourceCache,
    ast_cache: AstCache,
    ir_cache: IrCache,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    /// Creates a new runtime instance with empty caches.
    pub fn new() -> Self {
        Self {
            builtins: StdBuiltins::new(),
            source_cache: SourceCache::new(),
            ast_cache: AstCache::new(),
            ir_cache: IrCache::new(),
        }
    }

    /// Loads the AST for a model for debugging purposes.
    ///
    /// This method loads and parses a Oneil file, returning the AST node
    /// or a list of parsing errors.
    pub fn debug_load_ast(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<&debug::ast::ModelNode, Vec<OneilError>> {
        self.load_ast(&path);
        self.ast_cache
            .get_result(&path.as_ref().to_path_buf())
            .expect("should exist")
            .map_err(|errors| errors.to_vec())
    }

    /// Loads the IR for a model for debugging purposes.
    ///
    /// This method loads, parses, and resolves a Oneil file, returning the IR
    /// model collection or a list of errors.
    pub fn debug_load_ir(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<&IndexMap<debug::ir::ModelPath, debug::ir::Model>, Vec<OneilError>> {
        self.load_ir(path);

        let errors = self.get_all_errors();

        if errors.is_empty() {
            Ok(self.ir_cache.ir_collection())
        } else {
            Err(errors.to_vec())
        }
    }

    /// Loads the IR for a model, caching the result and reusing other caches.
    ///
    /// This method:
    /// 1. Checks the IR cache first
    /// 2. If not cached, loads the AST (which uses source and AST caches)
    /// 3. Converts the AST to IR using the model resolver
    /// 4. Caches the IR result
    fn load_ir(&mut self, path: impl AsRef<Path>) {
        let path = ir::ModelPath::new(path);

        // Check if IR is already cached - use `contains_model` to avoid borrow issues
        if self.ir_cache.contains_result(&path) {
            return;
        }

        // Use model resolver to convert AST to IR
        let ModelResolutionResult {
            models,
            model_errors,
            circular_dependency_errors,
        } = oneil_model_resolver::load_model(&path, self);

        // Insert the models into the cache
        self.ir_cache.insert_ir(models);

        // Handle the errors
        let circular_dependency_errors =
            self.convert_circular_dependency_errors(circular_dependency_errors);

        let resolution_errors = self.convert_resolution_errors(model_errors);

        self.ir_cache.insert_errors(circular_dependency_errors);
        self.ir_cache.insert_errors(resolution_errors);
    }

    fn convert_circular_dependency_errors(
        &self,
        circular_dependency_errors: IndexMap<
            ir::ModelPath,
            Vec<model_resolver::CircularDependencyError>,
        >,
    ) -> IndexMap<ir::ModelPath, Vec<OneilError>> {
        circular_dependency_errors
            .into_iter()
            .map(|(model_path, errors)| {
                (
                    model_path.clone(),
                    errors
                        .into_iter()
                        .map(|error| {
                            OneilError::from_error(&error, model_path.as_ref().to_path_buf())
                        })
                        .collect(),
                )
            })
            .collect()
    }

    fn convert_resolution_errors(
        &self,
        resolution_errors: IndexMap<ir::ModelPath, model_resolver::ResolutionErrors>,
    ) -> IndexMap<ir::ModelPath, Vec<OneilError>> {
        resolution_errors
            .into_iter()
            .map(|(model_path, resolution_errors)| {
                let model_path_buf = model_path.as_ref().to_path_buf();

                let mut errors = Vec::new();

                // Convert resolution errors to OneilErrors
                // Get source for location information
                let source = self
                    .source_cache
                    .get(&model_path_buf)
                    .expect("should exist")
                    .expect("should be have parsed correctly");

                // Convert each type of resolution error
                for import_error in resolution_errors.get_import_errors().values() {
                    errors.push(OneilError::from_error_with_source(
                        import_error,
                        model_path_buf.clone(),
                        source,
                    ));
                }

                for (_submodel_name, model_import_resolution_error) in resolution_errors
                    .get_model_import_resolution_errors()
                    .values()
                {
                    errors.push(OneilError::from_error_with_source(
                        model_import_resolution_error,
                        model_path_buf.clone(),
                        source,
                    ));
                }

                for parameter_errors in resolution_errors.get_parameter_resolution_errors().values()
                {
                    for parameter_error in parameter_errors {
                        errors.push(OneilError::from_error_with_source(
                            parameter_error,
                            model_path_buf.clone(),
                            source,
                        ));
                    }
                }

                for test_errors in resolution_errors.get_test_resolution_errors().values() {
                    for test_error in test_errors {
                        errors.push(OneilError::from_error_with_source(
                            test_error,
                            model_path_buf.clone(),
                            source,
                        ));
                    }
                }

                (model_path, errors)
            })
            .collect()
    }

    fn load_ast(&mut self, path: impl AsRef<Path>) {
        self.load_source(&path);

        let content = self
            .source_cache
            .get(&path.as_ref().to_path_buf())
            .expect("should exist");

        let Ok(content) = content else {
            return;
        };

        let parse_result = parser::parse_model(content, Some(Config::default()));

        match parse_result {
            Ok(ast) => {
                self.ast_cache.insert_ast(path.as_ref().to_path_buf(), ast);
            }
            Err(e) => {
                let errors = e
                    .errors
                    .into_iter()
                    .map(|e| OneilError::from_error(&e, path.as_ref().to_path_buf()))
                    .collect();

                self.ast_cache
                    .insert_errors(path.as_ref().to_path_buf(), errors);
            }
        }
    }

    fn load_source(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path);

        match content {
            Ok(content) => {
                self.source_cache.insert_source(path.to_path_buf(), content);
            }
            Err(error) => {
                let error = FileError::new(path, &error);
                let error = OneilError::from_error(&error, path.to_path_buf());
                self.source_cache.insert_error(path.to_path_buf(), error);
            }
        }
    }

    fn get_all_errors(&self) -> Vec<OneilError> {
        let mut errors = Vec::new();

        errors.extend(self.source_cache.get_all_errors());
        errors.extend(self.ast_cache.get_all_errors());
        errors.extend(self.ir_cache.get_all_errors());

        errors
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
    ) -> Result<&ast::ModelNode, model_resolver::AstLoadingFailedError> {
        let path = path.as_ref();
        let path_buf = path.to_path_buf();

        // Check if AST is already cached
        if self.ast_cache.contains_result(&path_buf) {
            let cached_result = self.ast_cache.get_result(&path_buf).expect("should exist");
            return cached_result.map_err(|_| model_resolver::AstLoadingFailedError);
        }

        self.load_ast(path);

        let cached_result = self.ast_cache.get_result(&path_buf).expect("should exist");
        cached_result.map_err(|_| model_resolver::AstLoadingFailedError)
    }

    fn load_python_import(
        &mut self,
        python_path: &ir::PythonPath,
    ) -> Result<(), model_resolver::PythonImportLoadingFailedError> {
        todo!()
    }
}
