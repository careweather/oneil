use std::path::Path;

use indexmap::IndexMap;
use oneil_ast as ast;
use oneil_ir as ir;
use oneil_model_resolver as model_resolver;
use oneil_parser::{self as parser, Config};
use oneil_shared::error::{AsOneilError, OneilError};

use crate::{
    cache::{AstCache, IrCache, SourceCache},
    debug,
    error::FileError,
    std_builtin::StdBuiltins,
};

/// Error type for Python import validation errors in the runtime
#[derive(Debug)]
pub struct PythonError(PathBuf);

impl std::fmt::Display for PythonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "python file '{}' does not exist", self.0.display())
    }
}

impl AsOneilError for PythonError {
    fn message(&self) -> String {
        format!("python file '{}' does not exist", self.0.display())
    }
}

use std::path::PathBuf;

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
        self.load_ast(&path)
    }

    /// Loads the IR for a model for debugging purposes.
    ///
    /// This method loads, parses, and resolves a Oneil file, returning the IR
    /// model collection or a list of errors.
    pub fn debug_load_ir(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<&IndexMap<debug::ir::ModelPath, debug::ir::Model>, Vec<OneilError>> {
        let _model = self.load_ir(path)?;
        Ok(self.ir_cache.ir_collection())
    }

    /// Loads the IR for a model, caching the result and reusing other caches.
    ///
    /// This method:
    /// 1. Checks the IR cache first
    /// 2. If not cached, loads the AST (which uses source and AST caches)
    /// 3. Converts the AST to IR using the model resolver
    /// 4. Caches the IR result
    /// 5. Returns the cached IR
    ///
    /// # Errors
    ///
    /// Returns errors if loading, parsing, or resolution fails.
    fn load_ir(&mut self, path: impl AsRef<Path>) -> Result<&ir::Model, Vec<OneilError>> {
        let path = ir::ModelPath::new(path);

        // Check if IR is already cached - use `contains_model` to avoid borrow issues
        if self.ir_cache.contains_result(&path) {
            // Now we can safely get it since we know it exists
            let cached_result = self.ir_cache.get_result(&path).expect("should exist");
            match cached_result {
                Ok(ir) => return Ok(ir),
                Err(errors) => return Err(errors.to_vec()),
            }
        }

        // Use model resolver to convert AST to IR
        // TODO: think about how to get rid of this clone
        let model_collection_result =
            oneil_model_resolver::load_model(&path, &self.builtins.clone(), self);

        match model_collection_result {
            Ok(model_collection) => {
                self.ir_cache.insert_ir(*model_collection);
                let ir = self.ir_cache.get_result(&path).expect("should exist");
                ir.map_err(|errors| errors.to_vec())
            }
            Err(error) => {
                let (partial_collection, error_map) = *error;

                self.ir_cache.insert_ir(partial_collection);

                let errors = self.convert_ir_errors(error_map);
                let errors = self.ir_cache.insert_errors(path, errors);
                let errors = errors.to_vec();

                Err(errors)
            }
        }
    }

    fn convert_ir_errors(
        &self,
        error_map: model_resolver::ModelErrorMap<Vec<OneilError>, PythonError>,
    ) -> Vec<OneilError> {
        // Convert model resolver errors to OneilErrors
        let mut errors: Vec<OneilError> = Vec::new();

        // Convert Python import errors first
        for (python_path, python_error) in error_map.get_import_errors() {
            let python_path_buf = python_path.as_ref().to_path_buf();
            // PythonError implements AsOneilError
            errors.push(OneilError::from_error(python_error, python_path_buf));
        }

        // Convert circular dependency errors
        for (model_path, circular_errors) in error_map.get_circular_dependency_errors() {
            let model_path_buf = model_path.as_ref().to_path_buf();
            for circular_error in circular_errors {
                errors.push(OneilError::from_error(
                    circular_error,
                    model_path_buf.clone(),
                ));
            }
        }

        // Convert model errors
        for (model_path, load_error) in error_map.get_model_errors() {
            let model_path_buf = model_path.as_ref().to_path_buf();
            match load_error {
                oneil_model_resolver::error::LoadError::ParseError(parse_errors) => {
                    errors.extend(parse_errors.iter().cloned());
                }
                oneil_model_resolver::error::LoadError::ResolutionErrors(resolution_errors) => {
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

                    for submodel_error in
                        resolution_errors.get_submodel_resolution_errors().values()
                    {
                        errors.push(OneilError::from_error_with_source(
                            submodel_error,
                            model_path_buf.clone(),
                            source,
                        ));
                    }

                    for reference_error in
                        resolution_errors.get_reference_resolution_errors().values()
                    {
                        errors.push(OneilError::from_error_with_source(
                            reference_error,
                            model_path_buf.clone(),
                            source,
                        ));
                    }

                    for parameter_errors in
                        resolution_errors.get_parameter_resolution_errors().values()
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
                }
            }
        }
        errors
    }

    fn load_ast(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<&debug::ast::ModelNode, Vec<OneilError>> {
        let content = self.load_source(&path).map_err(|e| vec![*e])?;
        let parse_result = parser::parse_model(content, Some(Config::default()));

        match parse_result {
            Ok(ast) => {
                let ast = self.ast_cache.insert_ast(path.as_ref().to_path_buf(), ast);

                Ok(ast)
            }
            Err(e) => {
                let errors = e
                    .errors
                    .into_iter()
                    .map(|e| OneilError::from_error(&e, path.as_ref().to_path_buf()))
                    .collect();

                let errors = self
                    .ast_cache
                    .insert_errors(path.as_ref().to_path_buf(), errors);

                Err(errors.to_vec())
            }
        }
    }

    fn load_source(&mut self, path: impl AsRef<Path>) -> Result<&str, Box<OneilError>> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path);

        match content {
            Ok(content) => {
                let content = self.source_cache.insert_source(path.to_path_buf(), content);
                Ok(content)
            }
            Err(error) => {
                let error = FileError::new(path, &error);
                let error = OneilError::from_error(&error, path.to_path_buf());
                let error = self.source_cache.insert_error(path.to_path_buf(), error);

                Err(Box::new(error))
            }
        }
    }
}

impl model_resolver::FileLoader for Runtime {
    type ParseError = Vec<OneilError>;
    type PythonError = PythonError;

    /// Parses a Oneil file into an AST using the cached source and AST.
    ///
    /// This method reuses the existing source and AST caches to avoid
    /// redundant file I/O and parsing operations. If the AST is not cached,
    /// it will parse the file (but won't cache it since this is a read-only operation).
    fn parse_ast(&mut self, path: impl AsRef<Path>) -> Result<ast::ModelNode, Self::ParseError> {
        let path = path.as_ref();
        let path_buf = path.to_path_buf();

        // Check if AST is already cached
        if let Some(cached_result) = self.ast_cache.get_result(&path_buf) {
            match cached_result {
                Ok(ast) => {
                    return Ok(ast.clone());
                }
                Err(errors) => {
                    return Err(errors.to_vec());
                }
            }
        }

        let ast = self.load_ast(path)?;

        Ok(ast.clone())
    }

    /// Validates a Python import by checking if the file exists.
    fn validate_python_import(&self, path: impl AsRef<Path>) -> Result<(), Self::PythonError> {
        let path = path.as_ref();
        if path.exists() {
            Ok(())
        } else {
            Err(PythonError(path.to_path_buf()))
        }
    }
}
