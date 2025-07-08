use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use crate::{debug_info::TraceLevel, expr::Expr, reference::Identifier};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestIndex(usize);

impl TestIndex {
    pub fn new(index: usize) -> Self {
        Self(index)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelTest {
    trace_level: TraceLevel,
    inputs: HashSet<Identifier>,
    test_expr: Expr,
}

impl ModelTest {
    pub fn new(trace_level: TraceLevel, inputs: HashSet<Identifier>, test_expr: Expr) -> Self {
        Self {
            trace_level,
            inputs,
            test_expr,
        }
    }

    pub fn trace_level(&self) -> &TraceLevel {
        &self.trace_level
    }

    pub fn inputs(&self) -> &HashSet<Identifier> {
        &self.inputs
    }

    pub fn test_expr(&self) -> &Expr {
        &self.test_expr
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubmodelTest {
    submodel_name: Identifier,
    inputs: SubmodelTestInputs,
}

impl SubmodelTest {
    pub fn new(submodel_name: Identifier, inputs: SubmodelTestInputs) -> Self {
        Self {
            submodel_name,
            inputs,
        }
    }

    pub fn submodel_name(&self) -> &Identifier {
        &self.submodel_name
    }

    pub fn inputs(&self) -> &SubmodelTestInputs {
        &self.inputs
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubmodelTestInputs(HashMap<Identifier, Expr>);

impl SubmodelTestInputs {
    pub fn new(inputs: HashMap<Identifier, Expr>) -> Self {
        Self(inputs)
    }
}

impl Deref for SubmodelTestInputs {
    type Target = HashMap<Identifier, Expr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
