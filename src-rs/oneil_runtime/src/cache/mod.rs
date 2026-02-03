//! Caches for source, AST, and IR used by the runtime.

mod ast;
mod ir;
mod source;

pub use ast::AstCache;
pub use ir::IrCache;
pub use source::SourceCache;
