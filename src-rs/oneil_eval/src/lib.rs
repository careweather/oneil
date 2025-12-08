// TODO: remove these allow statements
#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod builtin;
mod context;
mod error;
mod eval_expr;
mod eval_model;
mod eval_model_collection;
mod eval_parameter;
mod eval_unit;
mod result;
pub mod value;

pub use error::EvalError;
pub use eval_expr::eval_expr;
pub use eval_model::eval_model;
pub use eval_model_collection::eval_model_collection;
pub use eval_parameter::eval_parameter;
pub use eval_unit::eval_unit;
