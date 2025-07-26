use std::collections::HashSet;

use crate::store::StorageError;

pub type StorageResult<T> = Result<T, StorageError>;

/// A trait for a key-value storage backend with namespaced stores.
/// Keys are strings, values are Vec<String>.
pub trait KvStorage: Send + Sync {
    /// Initializes a named store (namespace). May be a no-op if already exists.
    fn initialize(&mut self, store: &str) -> StorageResult<()>;

    /// Retrieves a value by store name and key.
    fn get(&self, store: &str, key: &str) -> StorageResult<Vec<String>>;

    /// Sets a value for the given store and key.
    fn set(&mut self, store: &str, key: &str, value: Vec<String>) -> StorageResult<()>;
}
