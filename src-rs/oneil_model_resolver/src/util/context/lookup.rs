#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LookupResult<T> {
    Found(T),
    HasError,
    NotFound,
}

#[must_use]
pub fn lookup_with<K, V>(
    key: &K,
    lookup_value: impl Fn(&K) -> Option<V>,
    has_error: impl Fn(&K) -> bool,
) -> LookupResult<V> {
    if has_error(key) {
        LookupResult::HasError
    } else {
        lookup_value(key).map_or(LookupResult::NotFound, |value| LookupResult::Found(value))
    }
}
