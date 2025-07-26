use crate::store::{
    GenericDatabaseProvider, Result, StorageError, TransactionState,
};
use crate::GitopsResourceRoot;
use dashmap::DashMap;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::fs;
use tokio::io;

/// A filesystem-based implementation of `GenericDatabaseProvider`.
/// It now includes a configurable LRU cache to reduce file parsing overhead.
pub struct FilesystemDatabaseProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned + Clone,
{
    base_path: PathBuf,
    cache: Arc<DashMap<String, (T, SystemTime)>>,
    lru_keys: Arc<Mutex<VecDeque<String>>>,
    cache_capacity: usize,
    _phantom: PhantomData<T>,
}

impl<T> FilesystemDatabaseProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned + Clone,
{
    /// Creates a new `FilesystemDatabaseProvider`.
    ///
    /// * `base_path`: The root directory for storing resources.
    /// * `cache_capacity`: The number of parsed resources to keep in an in-memory LRU cache.
    pub fn new(base_path: impl Into<PathBuf>, cache_capacity: usize) -> Self {
        Self {
            base_path: base_path.into(),
            cache: Arc::new(DashMap::new()),
            lru_keys: Arc::new(Mutex::new(VecDeque::with_capacity(cache_capacity))),
            cache_capacity,
            _phantom: PhantomData,
        }
    }

    // -- Cache Helper Methods --

    /// Retrieves an item from the cache if it exists and moves it to the front of the LRU queue.
    fn cache_get(&self, key: &str) -> Option<(T, SystemTime)> {
        let result = self.cache.get(key).map(|r| r.value().clone());
        if result.is_some() {
            let mut lru = self.lru_keys.lock().unwrap();
            if let Some(pos) = lru.iter().position(|k| k == key) {
                if let Some(k) = lru.remove(pos) {
                    lru.push_front(k);
                }
            }
        }
        result
    }

    /// Inserts an item into the cache and updates the LRU queue, evicting the oldest item if capacity is exceeded.
    fn cache_insert(&self, key: String, item: T, modified: SystemTime) {
        if self.cache_capacity == 0 {
            return;
        }
        self.cache.insert(key.clone(), (item, modified));
        let mut lru = self.lru_keys.lock().unwrap();

        // Remove any existing instance of the key to prevent duplicates and update its freshness.
        if let Some(pos) = lru.iter().position(|k| *k == key) {
            lru.remove(pos);
        }

        lru.push_front(key);

        // Evict the least recently used item if the cache is over capacity.
        if lru.len() > self.cache_capacity {
            if let Some(key_to_evict) = lru.pop_back() {
                self.cache.remove(&key_to_evict);
            }
        }
    }

    /// Removes an item from the cache and its corresponding entry in the LRU queue.
    fn cache_remove(&self, key: &str) {
        self.cache.remove(key);
        let mut lru = self.lru_keys.lock().unwrap();
        if let Some(pos) = lru.iter().position(|k| k == key) {
            lru.remove(pos);
        }
    }

    // -- Core Logic --

    fn get_resource_path(&self, key: &str) -> PathBuf {
        self.base_path
            .join(T::kind())
            .join(format!("{}.yaml", urlencoding::encode(key)))
    }

    fn get_type_path(&self) -> PathBuf {
        self.base_path.join(T::kind())
    }

    async fn get_with_transaction_state(&self, key: &str) -> Result<(Option<(u, T)>, TransactionState)> {
        let path = self.get_resource_path(key);
        let map_io_err = |e: io::Error| StorageError::StorageError { reason: e.to_string() };

        match fs::metadata(&path).await {
            Ok(meta) => {
                let modified = meta.modified().map_err(map_io_err)?;
                // Check cache first
                if let Some((cached_item, cached_modified)) = self.cache_get(key) {
                    if cached_modified == modified {
                        return Ok((
                            Some(cached_item),
                            TransactionState::File {
                                path,
                                modified: Some(modified),
                            },
                        ));
                    }
                }

                // Cache miss or stale, read from file
                let content = fs::read_to_string(&path).await.map_err(map_io_err)?;
                let resource: T::Serializable = serde_yaml::from_str(&content).map_err(|e| {
                    StorageError::ReadItemFailure {
                        reason: e.to_string(),
                    }
                })?;
                let item = T::from(resource);

                self.cache_insert(key.to_string(), item.clone(), modified);

                Ok((
                    Some(item),
                    TransactionState::File {
                        path,
                        modified: Some(modified),
                    },
                ))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                self.cache_remove(key); // Ensure cache is clean if file is gone
                Ok((
                    None,
                    TransactionState::File {
                        path,
                        modified: None,
                    },
                ))
            }
            Err(e) => Err(map_io_err(e)),
        }
    }

    async fn write_with_transaction_state(
        &self,
        new_item: Option<&T>,
        state: &TransactionState,
    ) -> Result<()> {
        let (path, expected_modified_time) = match state {
            TransactionState::File { path, modified } => (path, modified),
            _ => return Err(StorageError::StorageError {
                reason: "Invalid transaction state for filesystem DB".to_string()
            }),
        };

        let map_io_err = |e: io::Error| StorageError::StorageError { reason: e.to_string() };

        if let Some(modified) = expected_modified_time {
            if path.exists() {
                let current_meta = fs::metadata(&path).await.map_err(map_io_err)?;
                if current_meta.modified().map_err(map_io_err)? != *modified {
                    return Err(StorageError::OptimisticLock);
                }
            } else {
                return Err(StorageError::OptimisticLock);
            }
        }

        match new_item {
            Some(item) => {
                let serializable_item = item.as_serializable();
                let yaml_content = serde_yaml::to_string(&serializable_item).map_err(|e| {
                    StorageError::WriteItemFailure {
                        reason: e.to_string(),
                    }
                })?;
                let parent_dir = path.parent().ok_or_else(|| StorageError::StorageError {
                    reason: format!("Failed to get parent directory for path: {:?}", path),
                })?;

                fs::create_dir_all(parent_dir).await.map_err(map_io_err)?;
                fs::write(&path, yaml_content).await.map_err(map_io_err)?;
            }
            None => {
                if expected_modified_time.is_some() {
                    match fs::remove_file(&path).await {
                        Ok(_) => {}
                        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                        Err(e) => return Err(map_io_err(e)),
                    }
                }
            }
        }
        Ok(())
    }
}

impl<T> GenericDatabaseProvider<T> for FilesystemDatabaseProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned + Clone,
{
    async fn list(&self) -> Result<Vec<(i64, T)>> {
        let keys = self.list_keys().await?;
        let mut resources = Vec::with_capacity(keys.len());
        for key in keys {
            if let Some(resource) = self.try_get_by_key(&key).await? {
                resources.push(resource);
            }
        }
        Ok(resources)
    }

    async fn list_keys(&self) -> Result<Vec<String>> {
        let dir_path = self.get_type_path();
        let mut keys = Vec::new();

        if !dir_path.exists() {
            return Ok(keys);
        }
        
        let map_io_err = |e: io::Error| StorageError::StorageError { reason: e.to_string() };
        let mut entries = fs::read_dir(&dir_path).await.map_err(map_io_err)?;

        while let Some(entry) = entries.next_entry().await.map_err(map_io_err)? {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml") {
                if let Some(stem) = path.file_stem() {
                    if let Some(key_str) = stem.to_str() {
                        let decoded_key = urlencoding::decode(key_str).map_err(|e| {
                            StorageError::ItemKeyError {
                                reason: e.to_string(),
                            }
                        })?;
                        keys.push(decoded_key.into_owned());
                    }
                }
            }
        }
        Ok(keys)
    }

    async fn get_by_key(&self, key: &str) -> Result<T> {
        self.try_get_by_key(key)
            .await?
            .ok_or_else(|| StorageError::ItemNotFound {
                key: key.to_string(),
                kind: T::kind().to_string(),
            })
    }

    async fn try_get_by_key(&self, key: &str) -> Result<Option<(T, i64)>> {
        self.get_with_transaction_state(key)
            .await
            .map(|(item, _)| item)
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let (_, tx_state) = self.get_with_transaction_state(key).await?;
        self.write_with_transaction_state(None, &tx_state).await?;
        self.cache_remove(key);
        Ok(())
    }

    async fn insert(&self, item: &T) -> Result<()> {
        let key = item.get_key();
        let (existing, tx_state) = self.get_with_transaction_state(&key).await?;

        if existing.is_some() {
            return Err(StorageError::Duplicate {
                key,
                kind: T::kind().to_string(),
            });
        }

        self.write_with_transaction_state(Some(item), &tx_state)
            .await?;
        self.cache_remove(&key);
        Ok(())
    }

    async fn upsert(&self, item: &T) -> Result<()> {
        let key = item.get_key();
        let (_, tx_state) = self.get_with_transaction_state(&key).await?;
        self.write_with_transaction_state(Some(item), &tx_state)
            .await?;
        self.cache_remove(&key);
        Ok(())
    }
}

// Namespaced provider implementation remains unchanged for now, but could be integrated
// with the new Store in a similar fashion if needed.
/// A generic database provider that supports namespaces.
pub trait GenericNamespacedDatabaseProvider<T>: Send + Sync
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned + Clone,
{
    async fn list(&self, ns: &str) -> Result<Vec<T>>;
    async fn list_keys(&self, ns: &str) -> Result<Vec<String>>;
    async fn get_by_key(&self, ns: &str, key: &str) -> Result<T>;
    async fn try_get_by_key(&self, ns: &str, key: &str) -> Result<Option<T>>;
    async fn delete(&self, ns: &str, key: &str) -> Result<()>;
    async fn insert(&self, ns: &str, item: &T) -> Result<()>;
    async fn upsert(&self, ns: &str, item: &T) -> Result<()>;
    async fn list_namespaces(&self) -> Result<Vec<String>>;
    async fn create_namespace(&self, ns: &str) -> Result<()>;
    async fn delete_namespace(&self, ns: &str, force: bool) -> Result<()>;
}

/// A filesystem-based implementation of `GenericNamespacedDatabaseProvider`.
pub struct FilesystemNamespacedDatabaseProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned + Clone,
{
    base_path: PathBuf,
    cache_capacity: usize,
    _phantom: PhantomData<T>,
}

impl<T> FilesystemNamespacedDatabaseProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned + Clone,
{
    pub fn new(base_path: impl Into<PathBuf>, cache_capacity: usize) -> Self {
        Self {
            base_path: base_path.into(),
            cache_capacity,
            _phantom: PhantomData,
        }
    }

    fn get_ns_path(&self, ns: &str) -> PathBuf {
        self.get_type_path()
            .join(urlencoding::encode(ns).as_ref())
    }
    
    fn get_type_path(&self) -> PathBuf {
        self.base_path.join(T::kind())
    }

    fn provider_for_namespace(&self, ns: &str) -> FilesystemDatabaseProvider<T> {
        FilesystemDatabaseProvider::new(self.get_ns_path(ns), self.cache_capacity)
    }
}

impl<T> GenericNamespacedDatabaseProvider<T> for FilesystemNamespacedDatabaseProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned + Clone,
{
    async fn list(&self, ns: &str) -> Result<Vec<T>> {
        self.provider_for_namespace(ns).list().await
    }

    async fn list_keys(&self, ns: &str) -> Result<Vec<String>> {
        self.provider_for_namespace(ns).list_keys().await
    }

    async fn get_by_key(&self, ns: &str, key: &str) -> Result<T> {
        self.provider_for_namespace(ns).get_by_key(key).await
    }

    async fn try_get_by_key(&self, ns: &str, key: &str) -> Result<Option<T>> {
        self.provider_for_namespace(ns).try_get_by_key(key).await
    }

    async fn delete(&self, ns: &str, key: &str) -> Result<()> {
        self.provider_for_namespace(ns).delete(key).await
    }

    async fn insert(&self, ns: &str, item: &T) -> Result<()> {
        self.provider_for_namespace(ns).insert(item).await
    }

    async fn upsert(&self, ns: &str, item: &T) -> Result<()> {
        let ns_path = self.get_ns_path(ns);
        if !ns_path.exists() {
            fs::create_dir_all(&ns_path)
                .await
                .map_err(|e| StorageError::StorageError { reason: e.to_string() })?;
        }
        self.provider_for_namespace(ns).upsert(item).await
    }

    async fn list_namespaces(&self) -> Result<Vec<String>> {
        let mut namespaces = Vec::new();
        let type_path = self.get_type_path();
        if !type_path.exists() {
            return Ok(namespaces);
        }
        let map_io_err = |e: io::Error| StorageError::StorageError { reason: e.to_string() };
        let mut entries = fs::read_dir(&type_path).await.map_err(map_io_err)?;
        while let Some(entry) = entries.next_entry().await.map_err(map_io_err)? {
            if entry.file_type().await.map_err(map_io_err)?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    let decoded_ns = urlencoding::decode(name).map_err(|e| {
                        StorageError::ItemKeyError {
                            reason: format!("Invalid namespace name format: {}", e),
                        }
                    })?;
                    namespaces.push(decoded_ns.into_owned());
                }
            }
        }
        Ok(namespaces)
    }

    async fn create_namespace(&self, ns: &str) -> Result<()> {
        fs::create_dir_all(self.get_ns_path(ns))
            .await
            .map_err(|e| StorageError::StorageError { reason: e.to_string() })
    }

    async fn delete_namespace(&self, ns: &str, force: bool) -> Result<()> {
        let ns_path = self.get_ns_path(ns);
        if !ns_path.exists() {
            return Ok(()); // Deleting a non-existent namespace is not an error.
        }
        
        let map_io_err = |e: io::Error| StorageError::StorageError { reason: e.to_string() };

        if !force {
            let mut entries = fs::read_dir(&ns_path).await.map_err(map_io_err)?;
            if entries.next_entry().await.map_err(map_io_err)?.is_some() {
                return Err(StorageError::StorageError {
                    reason: format!(
                        "Cannot delete non-empty namespace '{}' without 'force=true'",
                        ns
                    ),
                });
            }
        }

        fs::remove_dir_all(&ns_path).await.map_err(map_io_err)
    }
}

// Cloning a FilesystemDatabaseProvider should be possible for use across threads.
impl<T> Clone for FilesystemDatabaseProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned + Clone,
{
    fn clone(&self) -> Self {
        Self {
            base_path: self.base_path.clone(),
            cache: self.cache.clone(),
            lru_keys: self.lru_keys.clone(),
            cache_capacity: self.cache_capacity,
            _phantom: PhantomData,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
}

// Cloning a FilesystemNamespacedDatabaseProvider should be possible for use across threads.
impl<T> Clone for FilesystemNamespacedDatabaseProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned + Clone,
{
    fn clone(&self) -> Self {
        Self {
            base_path: self.base_path.clone(),
            cache_capacity: self.cache_capacity,
            _phantom: PhantomData,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
}
