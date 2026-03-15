//! TTL cache system for caching frequently accessed data.
//!
//! Each named cache is a key-value store with string keys and JSON values.
//! Entries expire after a configurable TTL. Access is thread-safe via `RwLock`.
//! Caches can optionally be bounded (max entries) — when full, expired entries
//! are evicted first, then the oldest entry is evicted.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::Value;
use tokio::sync::RwLock;

use crate::godmode;

pub const PRINCIPALS_CACHE: &str = "principals";
pub const PRINCIPALS_TTL: Duration = Duration::from_secs(5);

pub const PRINCIPAL_INFO_CACHE: &str = "principal_info";
pub const PRINCIPAL_INFO_TTL: Duration = Duration::from_secs(600); // 10 minutes
pub const PRINCIPAL_INFO_MAX_ENTRIES: usize = 2048;

/// A single cached entry with its insertion timestamp.
struct CacheEntry {
    value: Value,
    inserted_at: Instant,
}

/// A named TTL cache: string keys → JSON values, all sharing the same TTL.
/// Optionally bounded by `max_entries`.
struct TtlCache {
    entries: HashMap<String, CacheEntry>,
    ttl: Duration,
    max_entries: Option<usize>,
}

impl TtlCache {
    fn new(ttl: Duration, max_entries: Option<usize>) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
            max_entries,
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
        if let Some(max) = self.max_entries {
            // If we're at capacity and inserting a new key, evict.
            if self.entries.len() >= max && !self.entries.contains_key(&key) {
                self.evict(max);
            }
        }
        self.entries.insert(
            key,
            CacheEntry {
                value,
                inserted_at: Instant::now(),
            },
        );
    }

    /// Evict entries to make room. First purge all expired, then if still
    /// at/over `max`, remove the oldest entry.
    fn evict(&mut self, max: usize) {
        let now = Instant::now();
        // Purge expired
        self.entries
            .retain(|_, e| now.duration_since(e.inserted_at) < self.ttl);

        // Still over? Remove oldest
        if self.entries.len() >= max {
            if let Some(oldest_key) = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.inserted_at)
                .map(|(k, _)| k.clone())
            {
                self.entries.remove(&oldest_key);
            }
        }
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

    /// Ensure a named cache exists with the given TTL (unbounded).
    /// If the cache already exists, this is a no-op.
    pub async fn register_cache(&self, name: &str, ttl: Duration) {
        let mut caches = self.caches.write().await;
        caches
            .entry(name.to_string())
            .or_insert_with(|| TtlCache::new(ttl, None));
    }

    /// Ensure a named cache exists with TTL and a max entry count.
    /// When full, expired entries are purged first, then the oldest is evicted.
    pub async fn register_bounded_cache(&self, name: &str, ttl: Duration, max_entries: usize) {
        let mut caches = self.caches.write().await;
        caches
            .entry(name.to_string())
            .or_insert_with(|| TtlCache::new(ttl, Some(max_entries)));
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
    store.register_cache(PRINCIPALS_CACHE, PRINCIPALS_TTL).await;
    store
        .register_bounded_cache(
            PRINCIPAL_INFO_CACHE,
            PRINCIPAL_INFO_TTL,
            PRINCIPAL_INFO_MAX_ENTRIES,
        )
        .await;
    store
}
