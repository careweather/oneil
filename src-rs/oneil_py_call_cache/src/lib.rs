#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! Caching python function calls for the Oneil programming language.

mod error;
mod file;
mod function_call;

pub use error::{ReadCacheError, WriteCacheError};
pub use file::FileCache;
pub use function_call::{FunctionCall, FunctionCallResult};
