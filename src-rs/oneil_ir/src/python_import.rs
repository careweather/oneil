use indexmap::IndexSet;
use oneil_shared::span::Span;

use crate::PythonPath;

/// A name for a Python import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonImport {
    import_path: PythonPath,
    import_path_span: Span,
    functions: IndexSet<String>,
}

impl PythonImport {
    /// Creates a new Python import with the given path and span.
    #[must_use]
    pub const fn new(
        import_path: PythonPath,
        import_path_span: Span,
        functions: IndexSet<String>,
    ) -> Self {
        Self {
            import_path,
            import_path_span,
            functions,
        }
    }

    /// Returns the import path.
    #[must_use]
    pub const fn import_path(&self) -> &PythonPath {
        &self.import_path
    }

    /// Returns the span of the import path.
    #[must_use]
    pub const fn import_path_span(&self) -> &Span {
        &self.import_path_span
    }

    /// Returns the functions in the import.
    #[must_use]
    pub const fn functions(&self) -> &IndexSet<String> {
        &self.functions
    }
}
