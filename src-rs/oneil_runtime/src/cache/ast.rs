use std::{collections::HashMap, path::PathBuf};

use oneil_ast as ast;
use oneil_shared::error::OneilError;

pub struct AstCache {
    asts: HashMap<PathBuf, Result<ast::ModelNode, Vec<OneilError>>>,
}

impl AstCache {
    pub fn new() -> Self {
        Self {
            asts: HashMap::new(),
        }
    }

    pub fn insert_ast(&mut self, path: PathBuf, ast: ast::ModelNode) -> &ast::ModelNode {
        self.asts.insert(path.clone(), Ok(ast));

        self.asts
            .get(&path)
            .expect("ast should be in cache after insertion")
            .as_ref()
            .expect("ast should exist")
    }

    pub fn insert_errors(&mut self, path: PathBuf, errors: Vec<OneilError>) -> &[OneilError] {
        self.asts.insert(path.clone(), Err(errors));

        self.asts
            .get(&path)
            .expect("errors should be in cache after insertion")
            .as_ref()
            .expect_err("should be an error result")
    }

    pub fn get(&self, path: &PathBuf) -> Option<Result<&ast::ModelNode, &Vec<OneilError>>> {
        self.asts.get(path).map(|result| result.as_ref())
    }
}
