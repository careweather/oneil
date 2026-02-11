use std::path::PathBuf;

use indexmap::IndexMap;
use oneil_shared::error::OneilError;

#[derive(Debug)]
pub struct ErrorsCache {
    entries: IndexMap<PathBuf, ErrorEntry>,
}

impl ErrorsCache {
    pub fn new() -> Self {
        Self {
            entries: IndexMap::new(),
        }
    }

    pub fn insert_file_errors(&mut self, path: PathBuf, errors: Vec<OneilError>) {
        assert!(
            self.entries.get(&path).is_none(),
            "for now, we don't support inserting errors for the same path multiple times"
        );

        self.entries.insert(path, ErrorEntry::FileErrors(errors));
    }

    pub fn insert_model_errors(&mut self, path: PathBuf, errors: ModelErrors) {
        assert!(
            self.entries.get(&path).is_none(),
            "for now, we don't support inserting errors for the same path multiple times"
        );

        self.entries
            .insert(path, ErrorEntry::ModelErrors(Box::new(errors)));
    }
}

#[derive(Debug)]
pub enum ErrorEntry {
    FileErrors(Vec<OneilError>),
    ModelErrors(Box<ModelErrors>),
}

#[derive(Debug)]
pub struct ModelErrors {
    circular_dependency: Vec<OneilError>,
    model_import: IndexMap<PathBuf, Vec<OneilError>>,
    python_import: IndexMap<PathBuf, Vec<OneilError>>,
    parameter: IndexMap<String, Vec<OneilError>>,
    test: Vec<OneilError>,
}
