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

    pub fn insert_source(&mut self, path: PathBuf, content: String) -> &str {
        self.sources.insert(path.clone(), Ok(content));

        self.sources
            .get(&path)
            .expect("source should be in cache after insertion")
            .as_ref()
            .expect("source should exist")
    }

    pub fn insert_error(&mut self, path: PathBuf, error: OneilError) -> OneilError {
        self.sources.insert(path.clone(), Err(error));

        let error = self
            .sources
            .get(&path)
            .expect("error should be in cache after insertion")
            .as_ref()
            .expect_err("should be an error result");

        error.clone()
    }

    pub fn get(&self, path: &PathBuf) -> Option<Result<&str, &OneilError>> {
        self.sources
            .get(path)
            .map(|result| result.as_ref().map(|source| source.as_ref()))
    }
}
