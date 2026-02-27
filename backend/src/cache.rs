//! TTL cache system for caching frequently accessed data.
//!
//! Each named cache is a key-value store with string keys and JSON values.
//! Entries expire after a configurable TTL. Access is thread-safe via `RwLock`.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::Value;
use tokio::sync::RwLock;

use crate::godmode;

/// A single cached entry with its insertion timestamp.
struct CacheEntry {
    value: Value,
    inserted_at: Instant,
}

/// A named TTL cache: string keys → JSON values, all sharing the same TTL.
struct TtlCache {
    entries: HashMap<String, CacheEntry>,
    ttl: Duration,
}

impl TtlCache {
    fn new(ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
        }
    }

    fn get(&self, key: &str) -> Option<&Value> {
        let entry = self.entries.get(key)?;
        if entry.inserted_at.elapsed() < self.ttl {
            Some(&entry.value)
        } else {
            None
        }
    }

    fn set(&mut self, key: String, value: Value) {
        self.entries.insert(
            key,
            CacheEntry {
                value,
                inserted_at: Instant::now(),
            },
        );
    }

    fn invalidate(&mut self, key: &str) {
        self.entries.remove(key);
    }
}

/// Thread-safe container holding multiple named TTL caches.
/// Each cache is independently locked and has its own TTL.
pub struct CacheStore {
    caches: RwLock<HashMap<String, TtlCache>>,
}

impl CacheStore {
    pub fn new() -> Self {
        Self {
            caches: RwLock::new(HashMap::new()),
        }
    }

    /// Ensure a named cache exists with the given TTL.
    /// If the cache already exists, this is a no-op.
    pub async fn register_cache(&self, name: &str, ttl: Duration) {
        let mut caches = self.caches.write().await;
        caches
            .entry(name.to_string())
            .or_insert_with(|| TtlCache::new(ttl));
    }

    /// Get a value from a named cache. Returns `None` if the cache doesn't
    /// exist, the key is missing, or the entry has expired.
    pub async fn get(&self, cache_name: &str, key: &str) -> Option<Value> {
        let caches = self.caches.read().await;
        caches.get(cache_name).and_then(|c| c.get(key)).cloned()
    }

    /// Set a value in a named cache. The cache must have been registered first.
    pub async fn set(&self, cache_name: &str, key: String, value: Value) {
        let mut caches = self.caches.write().await;
        if let Some(cache) = caches.get_mut(cache_name) {
            cache.set(key, value);
        }
    }

    /// Remove a specific key from a named cache.
    pub async fn invalidate(&self, cache_name: &str, key: &str) {
        let mut caches = self.caches.write().await;
        if let Some(cache) = caches.get_mut(cache_name) {
            cache.invalidate(key);
        }
    }
}

/// Create a new `CacheStore` with the standard caches pre-registered.
pub async fn create_default_cache() -> Arc<CacheStore> {
    let store = Arc::new(CacheStore::new());
    store
        .register_cache(godmode::SPECIAL_ACCESS_CACHE, godmode::SPECIAL_ACCESS_TTL)
        .await;
    store
}
