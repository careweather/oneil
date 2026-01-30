use std::path::PathBuf;

use indexmap::IndexMap;
use oneil_ast as ast;
use oneil_shared::error::OneilError;

pub struct AstCache {
    asts: IndexMap<PathBuf, ast::ModelNode>,
    errors: IndexMap<PathBuf, Vec<OneilError>>,
}

impl AstCache {
    pub fn new() -> Self {
        Self {
            asts: IndexMap::new(),
            errors: IndexMap::new(),
        }
    }

    pub fn insert_ast(&mut self, path: PathBuf, ast: ast::ModelNode) {
        self.asts.insert(path.clone(), ast);
    }

    pub fn insert_errors(&mut self, path: PathBuf, errors: Vec<OneilError>) {
        self.errors.insert(path.clone(), errors);
    }

    pub fn get_result(&self, path: &PathBuf) -> Option<Result<&ast::ModelNode, &Vec<OneilError>>> {
        self.errors
            .get(path)
            .map(Err)
            .or_else(|| self.asts.get(path).map(Ok))
    }

    pub fn contains_result(&self, path: &PathBuf) -> bool {
        self.get_result(path).is_some()
    }

    pub fn get_ast(&self, path: &PathBuf) -> Option<&ast::ModelNode> {
        self.asts.get(path)
    }

    pub fn contains_ast(&self, path: &PathBuf) -> bool {
        self.get_ast(path).is_some()
    }

    pub fn get_errors(&self, path: &PathBuf) -> Option<&Vec<OneilError>> {
        self.errors.get(path)
    }

    pub fn contains_errors(&self, path: &PathBuf) -> bool {
        self.get_errors(path).is_some()
    }

    pub fn get_all_errors(&self) -> Vec<OneilError> {
        self.errors.values().flatten().cloned().collect()
    }
}
