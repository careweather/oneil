use std::collections::HashMap;

use oneil_ast as ast;

use crate::path::ModulePath;
use crate::reference::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct TestCollection {
    model_tests: Vec<Test>,
    dependency_tests: HashMap<ModulePath, TestInputs>,
}

impl TestCollection {
    pub fn new(model_tests: Vec<Test>, dependency_tests: HashMap<ModulePath, TestInputs>) -> Self {
        Self {
            model_tests,
            dependency_tests,
        }
    }

    pub fn empty() -> Self {
        Self::new(vec![], HashMap::new())
    }

    pub fn model_tests(&self) -> &Vec<Test> {
        &self.model_tests
    }

    pub fn dependency_tests(&self) -> &HashMap<ModulePath, TestInputs> {
        &self.dependency_tests
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    inputs: Vec<Identifier>,
    body: ast::Test,
}

impl Test {
    pub fn new(inputs: Vec<Identifier>, body: ast::Test) -> Self {
        Self { inputs, body }
    }

    pub fn inputs(&self) -> &Vec<Identifier> {
        &self.inputs
    }

    pub fn body(&self) -> &ast::Test {
        &self.body
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestIndex(usize);

impl TestIndex {
    pub fn new(index: usize) -> Self {
        Self(index)
    }
}
