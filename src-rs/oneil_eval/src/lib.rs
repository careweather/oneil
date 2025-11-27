// TODO: remove these allow statements
#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_variables)]

mod context;
mod error;
mod eval_expr;
mod eval_parameter;
mod eval_unit;
pub mod value;

pub use eval_expr::eval_expr;
pub use eval_parameter::eval_parameter;
pub use eval_unit::eval_unit;
