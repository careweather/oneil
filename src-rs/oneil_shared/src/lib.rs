#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! Shared utilities for the Oneil programming language

pub mod error;
pub mod load_result;
pub mod partial;
pub mod span;

pub use load_result::LoadResult;
