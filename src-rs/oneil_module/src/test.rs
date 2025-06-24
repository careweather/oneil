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
    pub fn new() -> Self {
        Self {
            model_tests: vec![],
            dependency_tests: HashMap::new(),
        }
    }

    pub fn add_model_test(&mut self, test: ast::Test) -> TestIndex {
        let test_index = self.model_tests.len();
        self.model_tests.push(test);
        TestIndex::new(test_index)
    }

    pub fn add_dependency_test(&mut self, module_path: ModulePath, inputs: TestInputs) {
        self.dependency_tests.insert(module_path, inputs);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestInputs(HashMap<Identifier, ast::Expr>);

impl TestInputs {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_input(&mut self, ident: Identifier, expr: ast::Expr) {
        self.0.insert(ident, expr);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestIndex(usize);

impl TestIndex {
    pub fn new(index: usize) -> Self {
        Self(index)
    }
}
