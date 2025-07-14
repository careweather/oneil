//! Helper structs for passing information about models, submodels, and parameters.
//!
//! This module provides the `InfoMap` type, which is used throughout the model
//! loading process to track both successful resolutions and items that have errors.
//! This allows resolution functions to make informed decisions about error handling
//! and provides comprehensive error reporting.

use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

/// A map that tracks both successful lookups and items with errors.
///
/// `InfoMap` is used during model resolution to track information about models,
/// submodels, and parameters. It maintains two collections:
///
/// - A map of successfully resolved items
/// - A set of items that have errors
///
/// This dual tracking allows resolution functions to distinguish between items
/// that don't exist and items that exist but have errors, enabling better
/// error reporting and recovery strategies.
#[derive(Debug, Clone)]
pub struct InfoMap<K, V>
where
    K: Eq + Hash,
    V: Debug + Clone,
{
    /// Map of successfully resolved items.
    pub map: HashMap<K, V>,
    /// Set of items that have errors.
    pub with_errors: HashSet<K>,
}

impl<K, V> InfoMap<K, V>
where
    K: Eq + Hash,
    V: Debug + Clone,
{
    /// Creates a new `InfoMap` with the specified successful items and error items.
    ///
    /// # Arguments
    ///
    /// * `map` - HashMap of successfully resolved items
    /// * `with_errors` - HashSet of items that have errors
    ///
    /// # Returns
    ///
    /// A new `InfoMap` instance.
    pub fn new(map: HashMap<K, V>, with_errors: HashSet<K>) -> Self {
        Self { map, with_errors }
    }

    /// Looks up an item by key, returning information about its status.
    ///
    /// This method checks both the successful items map and the error set to
    /// determine the status of the requested item.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up (can be borrowed from any type that implements `Borrow<K>`)
    ///
    /// # Returns
    ///
    /// Returns an `InfoResult` indicating the status of the item:
    /// - `Found(value)` if the item exists and has no errors
    /// - `HasError` if the item exists but has errors
    /// - `NotFound` if the item doesn't exist
    pub fn get(&self, key: impl Borrow<K>) -> InfoResult<&V> {
        if self.with_errors.contains(key.borrow()) {
            InfoResult::HasError
        } else {
            match self.map.get(key.borrow()) {
                Some(value) => InfoResult::Found(value),
                None => InfoResult::NotFound,
            }
        }
    }
}

/// Result of looking up an item in an `InfoMap`.
///
/// This enum represents the three possible states when looking up an item:
/// found successfully, found but with errors, or not found at all.
#[derive(Debug, Clone)]
pub enum InfoResult<T>
where
    T: Debug + Clone,
{
    /// The item was found successfully and has no errors.
    Found(T),
    /// The item exists but has errors.
    HasError,
    /// The item was not found.
    NotFound,
}
