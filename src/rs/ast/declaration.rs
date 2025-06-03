use super::expression::Expr;
use super::note::Note;
use super::parameter::{Limits, ParameterValue, TraceLevel};

#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    Import {
        path: String,
    },
    Use {
        from_model: Option<String>,
        use_model: String,
        inputs: Option<Vec<ModelInput>>,
        as_name: String,
    },
    Parameter {
        name: String,
        ident: String,
        value: ParameterValue,
        limits: Limits,
        is_performance: bool,
        trace_level: TraceLevel,
        note: Option<Note>,
    },
    Test {
        trace_level: TraceLevel,
        inputs: Vec<String>,
        expr: Expr,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelInput {
    pub name: String,
    pub value: Expr,
}
