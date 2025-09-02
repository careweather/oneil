#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LookupResult<T> {
    Found(T),
    HasError,
    NotFound,
}

pub fn lookup_with<K, V>(
    key: &K,
    lookup_value: impl Fn(&K) -> Option<V>,
    has_error: impl Fn(&K) -> bool,
) -> LookupResult<V> {
    if has_error(key) {
        LookupResult::HasError
    } else {
        match lookup_value(key) {
            Some(value) => LookupResult::Found(value),
            None => LookupResult::NotFound,
        }
    }
}
