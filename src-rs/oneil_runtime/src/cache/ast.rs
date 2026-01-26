use std::{collections::HashMap, path::PathBuf};

use oneil_ast as ast;
use oneil_shared::error::OneilError;

pub struct AstCache {
    sources: HashMap<PathBuf, Result<ast::ModelNode, Vec<OneilError>>>,
}

impl AstCache {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    pub fn insert_ast(&mut self, path: PathBuf, ast: ast::ModelNode) -> &ast::ModelNode {
        self.sources.insert(path.clone(), Ok(ast));
        let ast = self
            .sources
            .get(&path)
            .expect("ast should be in cache after insertion")
            .as_ref()
            .expect("ast should exist");

        &ast
    }

    pub fn insert_errors(&mut self, path: PathBuf, errors: Vec<OneilError>) -> &[OneilError] {
        self.sources.insert(path.clone(), Err(errors));
        let errors = self
            .sources
            .get(&path)
            .expect("errors should be in cache after insertion")
            .as_ref()
            .expect_err("should be an error result");

        &errors
    }

    pub fn get(&self, path: &PathBuf) -> Option<Result<&ast::ModelNode, &Vec<OneilError>>> {
        self.sources
            .get(path)
            .map(|result| result.as_ref().map(|ast| ast))
    }
}
