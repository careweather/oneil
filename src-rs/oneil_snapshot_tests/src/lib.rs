//! Snapshot testing for Oneil evaluation output and errors.
//!
//! This crate provides integration-style snapshot tests that run the full
//! Oneil pipeline (parse -> resolve -> eval) and capture evaluation output
//! and errors in a canonical format for comparison.

#[cfg(test)]
mod test;

#[cfg(test)]
mod util;
