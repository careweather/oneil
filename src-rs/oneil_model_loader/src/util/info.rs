//! Helper structs for passing information about modules, submodels, and
//! parameters

use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

#[derive(Debug, Clone)]
pub struct InfoMap<K, V>
where
    K: Eq + Hash,
    V: Debug + Clone,
{
    pub map: HashMap<K, V>,
    pub with_errors: HashSet<K>,
}

impl<K, V> InfoMap<K, V>
where
    K: Eq + Hash,
    V: Debug + Clone,
{
    pub fn new(map: HashMap<K, V>, with_errors: HashSet<K>) -> Self {
        Self { map, with_errors }
    }

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

#[derive(Debug, Clone)]
pub enum InfoResult<T>
where
    T: Debug + Clone,
{
    Found(T),
    HasError,
    NotFound,
}
