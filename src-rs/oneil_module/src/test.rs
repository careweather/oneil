use std::collections::HashMap;

use oneil_ast as ast;

use crate::path::ModulePath;
use crate::reference::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct Tests {
    model_tests: Vec<ast::Test>,
    dependency_tests: HashMap<ModulePath, TestInputs>,
}

impl Tests {
    pub fn new(
        model_tests: Vec<ast::Test>,
        dependency_tests: HashMap<ModulePath, TestInputs>,
    ) -> Self {
        Self {
            model_tests,
            dependency_tests,
        }
    }

    pub fn empty() -> Self {
        Self::new(vec![], HashMap::new())
    }

    pub fn model_tests(&self) -> &Vec<ast::Test> {
        &self.model_tests
    }

    pub fn dependency_tests(&self) -> &HashMap<ModulePath, TestInputs> {
        &self.dependency_tests
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestInputs(HashMap<Identifier, ast::Expr>);

impl TestInputs {
    pub fn new(inputs: HashMap<Identifier, ast::Expr>) -> Self {
        Self(inputs)
    }

    pub fn empty() -> Self {
        Self::new(HashMap::new())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestIndex(usize);

impl TestIndex {
    pub fn new(index: usize) -> Self {
        Self(index)
    }
}
