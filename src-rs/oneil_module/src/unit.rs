#[derive(Debug, Clone, PartialEq)]
pub struct CompositeUnit {
    units: Vec<Unit>,
}

impl CompositeUnit {
    pub fn new(units: Vec<Unit>) -> Self {
        Self { units }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    name: String,
    exponent: f64,
}

impl Unit {
    pub fn new(name: String, exponent: f64) -> Self {
        Self { name, exponent }
    }
}
