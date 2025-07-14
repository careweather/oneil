//! Provides token parsing functionality for the Oneil language.
//!
//! This module contains parsers for all token types in the Oneil language, organized into
//! submodules by token category:
//!
//! - `keyword`: Reserved language keywords (e.g., `if`, `and`, `import`)
//! - `literal`: Literal values (e.g., numbers, strings)
//! - `naming`: Identifiers and names
//! - `note`: Single-line (`~`) and multi-line (`~~~`) notes
//! - `structure`: Structural elements like line endings and comments
//! - `symbol`: Operators and special characters
//!
//! Each submodule provides specialized parsers that handle the specific token types
//! while following consistent patterns for whitespace handling and error reporting.
//!
//! All token parsers consume trailing whitespace after the matched content and
//! return the matched content as a `Span`.

use super::util::{Parser, Result, Span};

pub mod error;
mod util;
pub use util::Token;

pub mod keyword;
pub mod literal;
pub mod naming;
pub mod note;
pub mod structure;
pub mod symbol;
