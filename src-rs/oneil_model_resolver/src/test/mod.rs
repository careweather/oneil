//! Helper functions for creating test data
//!
//! Creating test data can be a tedious and repetitive process, especially where `Span`s are
//! involved. This module provides helper functions to create test data that can be used in tests.

use oneil_shared::span::Span;

pub mod external_context;
pub mod resolution_context;
pub mod test_ast;
pub mod test_ir;

pub fn unimportant_span() -> Span {
    Span::random_span()
}
