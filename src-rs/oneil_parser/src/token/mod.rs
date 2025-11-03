//! Provides token parsing functionality for the Oneil language.

use super::util::{InputSpan, Parser, Result};

pub mod error;
mod util;

pub mod keyword;
pub mod literal;
pub mod naming;
pub mod note;
pub mod structure;
pub mod symbol;
