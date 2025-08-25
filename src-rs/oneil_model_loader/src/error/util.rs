//! Utility functions for error handling and manipulation.
//!
//! This module provides utility functions for working with errors in a functional
//! programming style. These functions help with combining errors from multiple
//! operations, converting between error types, and separating successful results
//! from errors.
//!
//! # Key Functions
//!
//! - `combine_errors`: Combines errors from two operations
//! - `combine_error_list`: Combines errors from a list of operations
//! - `split_ok_and_errors`: Separates successful results from errors
//! - `convert_errors`: Converts errors from one type to another

use std::{collections::HashMap, hash::Hash};

/// Combines the results of two operations, collecting all errors.
///
/// This function takes two `Result` values and combines them. If both operations
/// succeed, it returns `Ok((result1, result2))`. If either operation fails, it
/// returns `Err(errors)` containing all the errors from the failed operations.
///
/// # Arguments
///
/// * `result1` - The result of the first operation
/// * `result2` - The result of the second operation
///
/// # Returns
///
/// Returns `Ok((T, U))` if both operations succeed, or `Err(Vec<E>)` if either
/// operation fails, containing all errors from the failed operations.
#[allow(
    clippy::missing_errors_doc,
    reason = "this is a utility function for merging errors"
)]
pub fn combine_errors<T, U, E>(
    result1: Result<T, Vec<E>>,
    result2: Result<U, Vec<E>>,
) -> Result<(T, U), Vec<E>> {
    match (result1, result2) {
        (Ok(result1), Ok(result2)) => Ok((result1, result2)),
        (Err(errors), Ok(_)) | (Ok(_), Err(errors)) => Err(errors),
        (Err(errors1), Err(errors2)) => Err(errors1.into_iter().chain(errors2).collect()),
    }
}

/// Combines the results of multiple operations, collecting all errors.
///
/// This function takes an iterator of `Result` values and combines them. If all
/// operations succeed, it returns `Ok(Vec<T>)` containing all successful results.
/// If any operation fails, it returns `Err(Vec<E>)` containing all errors from
/// all failed operations.
///
/// # Arguments
///
/// * `results` - An iterator of results to combine
///
/// # Returns
///
/// Returns `Ok(Vec<T>)` if all operations succeed, or `Err(Vec<E>)` if any
/// operation fails, containing all errors from all failed operations.
#[allow(
    clippy::missing_errors_doc,
    reason = "this is a utility function for merging errors"
)]
pub fn combine_error_list<T, E>(
    results: impl IntoIterator<Item = Result<T, Vec<E>>>,
) -> Result<Vec<T>, Vec<E>> {
    #[allow(
        clippy::manual_try_fold,
        reason = "we want to consume *all* errors, not just the first one"
    )]
    results
        .into_iter()
        .fold(Ok(Vec::new()), |acc, result| match acc {
            Ok(mut acc) => match result {
                Ok(result) => {
                    acc.push(result);
                    Ok(acc)
                }
                Err(errors) => Err(errors),
            },
            Err(mut acc_errors) => match result {
                Ok(_result) => Err(acc_errors),
                Err(errors) => {
                    acc_errors.extend(errors);
                    Err(acc_errors)
                }
            },
        })
}

/// Separates successful results from errors in a collection of results.
///
/// This function takes an iterator of `Result` values where the error type is
/// a tuple of a key and a list of errors. It separates the successful results
/// from the errors, returning both collections.
///
/// # Arguments
///
/// * `results` - An iterator of results to separate
///
/// # Returns
///
/// Returns a tuple `(O, HashMap<I, Vec<E>>)` where:
/// - `O` is a collection of successful results (must implement `FromIterator<T>`)
/// - `HashMap<I, Vec<E>>` maps error keys to their associated errors
///
/// # Panics
///
/// Panics if there are duplicate error keys in the results.
#[must_use]
pub fn split_ok_and_errors<T, I, E, O>(
    results: impl IntoIterator<Item = Result<T, (I, Vec<E>)>>,
) -> (O, HashMap<I, Vec<E>>)
where
    I: Eq + Hash,
    O: FromIterator<T>,
{
    let (ok, errors) = results.into_iter().fold(
        (Vec::new(), HashMap::new()),
        |(mut ok, mut acc_errors), result| match result {
            Ok(result) => {
                ok.push(result);
                (ok, acc_errors)
            }
            Err((key, errors)) => {
                assert!(!acc_errors.contains_key(&key), "duplicate error");
                acc_errors.insert(key, errors);
                (ok, acc_errors)
            }
        },
    );

    let ok = ok.into_iter().collect();
    let errors = errors.into_iter().collect();

    (ok, errors)
}

/// Converts a vector of errors from one type to another.
///
/// This function takes a vector of errors and converts each error to a new type
/// using the `Into` trait. This is useful when you need to convert between
/// different error types in a collection.
///
/// # Arguments
///
/// * `errors` - A vector of errors to convert
///
/// # Returns
///
/// Returns a vector of converted errors.
#[must_use]
pub fn convert_errors<E1, E2>(errors: Vec<E1>) -> Vec<E2>
where
    E1: Into<E2>,
{
    errors.into_iter().map(Into::into).collect()
}
