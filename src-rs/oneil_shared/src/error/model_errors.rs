use std::path::PathBuf;

use indexmap::IndexMap;

/// Collection of errors that occurred in a model.
#[derive(Debug, Clone)]
pub struct ModelErrors<E> {
    /// The errors that occurred during evaluation of the Python imports.
    pub python_imports: IndexMap<PathBuf, Vec<E>>,
    /// The errors that occurred during evaluation of the model imports.
    pub model_imports: IndexMap<PathBuf, Vec<E>>,
    /// The errors that occurred during evaluation of the parameters.
    pub parameters: IndexMap<String, Vec<E>>,
    /// The errors that occurred during evaluation of the tests.
    pub tests: Vec<E>,
}

impl<E> ModelErrors<E> {
    /// Maps the errors to a new type.
    pub fn map<E2, F>(self, f: F) -> ModelErrors<E2>
    where
        F: Fn(E) -> E2,
    {
        ModelErrors {
            python_imports: self
                .python_imports
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().map(&f).collect()))
                .collect(),

            model_imports: self
                .model_imports
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().map(&f).collect()))
                .collect(),

            parameters: self
                .parameters
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().map(&f).collect()))
                .collect(),

            tests: self.tests.into_iter().map(&f).collect(),
        }
    }

    /// Returns an iterator over all errors (flattened)
    pub fn iter(&self) -> impl Iterator<Item = &E> {
        let python_imports_iter = self.python_imports.values().flatten();
        let model_imports_iter = self.model_imports.values().flatten();
        let parameters_iter = self.parameters.values().flatten();
        let tests_iter = self.tests.iter();

        python_imports_iter
            .chain(model_imports_iter)
            .chain(parameters_iter)
            .chain(tests_iter)
    }
}
