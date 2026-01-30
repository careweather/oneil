use std::{collections::HashMap, path::PathBuf};

use oneil_shared::error::OneilError;

pub struct SourceCache {
    sources: HashMap<PathBuf, Result<String, OneilError>>,
}

impl SourceCache {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    pub fn insert_source(&mut self, path: PathBuf, content: String) {
        self.sources.insert(path, Ok(content));
    }

    pub fn insert_error(&mut self, path: PathBuf, error: OneilError) {
        self.sources.insert(path, Err(error));
    }

    pub fn get(&self, path: &PathBuf) -> Option<Result<&str, &OneilError>> {
        self.sources
            .get(path)
            .map(|result| result.as_ref().map(|source| source.as_ref()))
    }

    pub fn get_all_errors(&self) -> Vec<OneilError> {
        self.sources
            .values()
            .filter_map(|result| result.as_ref().err())
            .cloned()
            .collect()
    }
}
