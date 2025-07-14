#[derive(Debug, Clone, PartialEq)]
pub struct Unit;

impl Unit {
    pub fn empty() -> Self {
        Self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestUnits;

#[derive(Debug, Clone, PartialEq)]
pub struct SubmodelTestUnits;
