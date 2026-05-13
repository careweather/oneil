//! Errors for cache file I/O.

use std::fmt;

/// Failure when writing a cache JSON file.
#[derive(Debug)]
pub enum WriteCacheError {
    /// I/O error creating or writing the file.
    Io(std::io::Error),
    /// JSON serialization error.
    Serde(serde_json::Error),
}

impl fmt::Display for WriteCacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "failed to write cache file: {e}"),
            Self::Serde(e) => write!(f, "failed to serialize cache to JSON: {e}"),
        }
    }
}

impl std::error::Error for WriteCacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Serde(e) => Some(e),
        }
    }
}

/// Failure when reading a cache JSON file.
#[derive(Debug)]
pub enum ReadCacheError {
    /// I/O error opening or reading the file.
    Io(std::io::Error),
    /// JSON deserialization error.
    Serde(serde_json::Error),
}

impl fmt::Display for ReadCacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "failed to read cache file: {e}"),
            Self::Serde(e) => write!(f, "failed to deserialize cache from JSON: {e}"),
        }
    }
}

impl std::error::Error for ReadCacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Serde(e) => Some(e),
        }
    }
}
