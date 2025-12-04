use crate::value::Unit;

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Boolean,
    String,
    Number { unit: Unit, number_type: NumberType },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberType {
    Scalar,
    Interval,
}
