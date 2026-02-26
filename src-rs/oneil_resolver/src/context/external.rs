use indexmap::IndexSet;
use oneil_ast as ast;
use oneil_ir as ir;
use oneil_shared::load_result::LoadResult;

/// Error indicating that loading/parsing a model's AST failed.
pub struct AstLoadingFailedError;

/// Error indicating that loading a Python import failed.
pub struct PythonImportLoadingFailedError;

/// Context provided by the environment for resolving models (builtins, AST loading, Python imports).
pub trait ExternalResolutionContext {
    /// Checks if the given identifier refers to a builtin value.
    fn has_builtin_value(&self, identifier: &ir::Identifier) -> bool;

    /// Checks if the given identifier refers to a builtin function.
    fn has_builtin_function(&self, identifier: &ir::Identifier) -> bool;

    /// Checks if the given name refers to a builtin unit.
    fn has_builtin_unit(&self, name: &str) -> bool;

    /// Returns the available unit prefixes (e.g., "k" -> 1000.0).
    fn available_prefixes(&self) -> impl Iterator<Item = (&str, f64)>;

    /// Returns whether the given unit name supports SI prefixes.
    fn unit_supports_si_prefixes(&self, name: &str) -> bool;

    /// Loads the AST for a model.
    ///
    /// # Errors
    ///
    /// Returns `Err(AstLoadingFailedError)` when the model file cannot be read or parsed.
    fn load_ast(
        &mut self,
        path: &ir::ModelPath,
    ) -> LoadResult<&ast::ModelNode, AstLoadingFailedError>;

    /// Loads a Python import.
    ///
    /// # Errors
    ///
    /// Returns `Err(PythonImportLoadingFailedError)` when the Python import cannot be loaded or
    /// validated.
    fn load_python_import<'context>(
        &'context mut self,
        python_path: &ir::PythonPath,
    ) -> Result<IndexSet<&'context str>, PythonImportLoadingFailedError>;
}
