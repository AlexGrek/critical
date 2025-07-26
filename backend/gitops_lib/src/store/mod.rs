use crate::GitopsResourceRoot;
use crate::store::filesystem::{
    FilesystemDatabaseProvider, FilesystemNamespacedDatabaseProvider,
    GenericNamespacedDatabaseProvider,
};
use dashmap::DashMap;
use serde::{de::DeserializeOwned, Serialize};
use std::any::{Any, TypeId};
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;
pub mod config;
pub mod filesystem;
pub mod qstorage;
pub mod qstorage_persy;
pub mod qstorage_sled;
use config::{BackendConfig, StoreConfig};

/// A specialized Result type for storage operations.
pub type Result<T, E = StorageError> = std::result::Result<T, E>;

/// Defines common errors that can occur during storage operations.
#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("Item with key '{key}' of kind '{kind}' not found")]
    ItemNotFound { key: String, kind: String },

    #[error("Failed to read/deserialize item: {reason}")]
    ReadItemFailure { reason: String },

    #[error("Failed to write/serialize item: {reason}")]
    WriteItemFailure { reason: String },

    #[error("Item with key '{key}' of kind '{kind}' already exists")]
    Duplicate { key: String, kind: String },

    #[error("Namespace '{ns}' not found")]
    NamespaceNotFound { ns: String },

    #[error("The provided item key is invalid: {reason}")]
    ItemKeyError { reason: String },

    #[error("A generic storage error occurred: {reason}")]
    StorageError { reason: String },

    #[error("Optimistic lock failed: resource was modified by another process")]
    OptimisticLock,
}

/// A type-erased, dynamically-dispatchable database provider for a specific resource `T`.
///
/// This enum wraps concrete provider implementations, allowing the `Store` to
/// return a single type that can represent any configured backend (Filesystem, Sqlite, etc.).
/// It implements the `GenericDatabaseProvider` trait by dispatching calls to the wrapped variant.
pub enum AnyProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned,
{
    Filesystem(FilesystemDatabaseProvider<T>),
    // When you add a Sqlite provider, you would add a variant here:
    // Sqlite(SqliteDatabaseProvider<T>),
}

impl<T> GenericDatabaseProvider<T> for AnyProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned,
{
    async fn list(&self) -> Result<Vec<T>> {
        match self {
            AnyProvider::Filesystem(p) => p.list().await,
        }
    }

    async fn list_keys(&self) -> Result<Vec<String>> {
        match self {
            AnyProvider::Filesystem(p) => p.list_keys().await,
        }
    }

    async fn get_by_key(&self, key: &str) -> Result<T> {
        match self {
            AnyProvider::Filesystem(p) => p.get_by_key(key).await,
        }
    }

    async fn try_get_by_key(&self, key: &str) -> Result<Option<T>> {
        match self {
            AnyProvider::Filesystem(p) => p.try_get_by_key(key).await,
        }
    }

    async fn delete(&self, key: &str) -> Result<()> {
        match self {
            AnyProvider::Filesystem(p) => p.delete(key).await,
        }
    }

    async fn insert(&self, item: &T) -> Result<()> {
        match self {
            AnyProvider::Filesystem(p) => p.insert(item).await,
        }
    }

    async fn upsert(&self, item: &T) -> Result<()> {
        match self {
            AnyProvider::Filesystem(p) => p.upsert(item).await,
        }
    }
}

pub enum AnyNsProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned,
{
    Filesystem(FilesystemNamespacedDatabaseProvider<T>),
    // When you add a Sqlite provider, you would add a variant here:
    // Sqlite(SqliteDatabaseProvider<T>),
}

impl<T> GenericNamespacedDatabaseProvider<T> for AnyNsProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned,
{
    async fn list(&self, ns: &str) -> Result<Vec<T>> {
        match self {
            AnyNsProvider::Filesystem(p) => p.list(ns).await,
        }
    }

    async fn list_keys(&self, ns: &str) -> Result<Vec<String>> {
        match self {
            AnyNsProvider::Filesystem(p) => p.list_keys(ns).await,
        }
    }

    async fn get_by_key(&self, ns: &str, key: &str) -> Result<T> {
        match self {
            AnyNsProvider::Filesystem(p) => p.get_by_key(ns, key).await,
        }
    }

    async fn try_get_by_key(&self, ns: &str, key: &str) -> Result<Option<T>> {
        match self {
            AnyNsProvider::Filesystem(p) => p.try_get_by_key(ns, key).await,
        }
    }

    async fn delete(&self, ns: &str, key: &str) -> Result<()> {
        match self {
            AnyNsProvider::Filesystem(p) => p.delete(ns, key).await,
        }
    }

    async fn insert(&self, ns: &str, item: &T) -> Result<()> {
        match self {
            AnyNsProvider::Filesystem(p) => p.insert(ns, item).await,
        }
    }

    async fn upsert(&self, ns: &str, item: &T) -> Result<()> {
        match self {
            AnyNsProvider::Filesystem(p) => p.upsert(ns, item).await,
        }
    }

    async fn list_namespaces(&self) -> Result<Vec<String>> {
        match self {
            AnyNsProvider::Filesystem(p) => p.list_namespaces().await,
        }
    }

    async fn create_namespace(&self, ns: &str) -> Result<()> {
        match self {
            AnyNsProvider::Filesystem(p) => p.create_namespace(ns).await,
        }
    }

    async fn delete_namespace(&self, ns: &str, force: bool) -> Result<()> {
        match self {
            AnyNsProvider::Filesystem(p) => p.delete_namespace(ns, force).await,
        }
    }
}

/// A factory for creating database providers based on a runtime configuration.
///
/// This struct is designed to be cloned and shared in application state (e.g., Axum).
/// It caches provider instances to avoid re-creating them on every request.
#[derive(Clone)]
pub struct Store {
    config: Arc<StoreConfig>,
    /// Caches instantiated providers. Key is `TypeId` of the resource, Value is `Arc<dyn Any + Send + Sync>`.
    /// The `dyn Any` value is a downcastable `Arc<AnyProvider<T>>`.
    providers: Arc<DashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    providers_ns: Arc<DashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl Store {
    /// Creates a new `Store` with the given configuration.
    pub fn new(config: StoreConfig) -> Self {
        Self {
            config: Arc::new(config),
            providers: Arc::new(DashMap::new()),
            providers_ns: Arc::new(DashMap::new()),
        }
    }

    /// Returns a provider for a specific, non-namespaced resource type `T`.
    ///
    /// This method uses a cache to ensure that only one provider instance is created
    /// for each resource type. It determines which backend to use based on the
    /// configuration provided at `Store` creation.
    pub fn provider<T>(&self) -> Arc<AnyProvider<T>>
    where
        T: GitopsResourceRoot + Serialize + DeserializeOwned,
    {
        let type_id = TypeId::of::<T>();

        // Fast path: check if the provider is already cached.
        if let Some(entry) = self.providers.get(&type_id) {
            return entry.value().clone().downcast::<AnyProvider<T>>().unwrap();
        }

        // Slow path: not in cache, so create, insert, and return.
        let resource_kind = T::kind();
        let backend_config = self
            .config
            .resource_backends
            .get(resource_kind)
            .or(self.config.default_backend.as_ref())
            .unwrap_or_else(|| {
                panic!(
                    "No backend configured for kind '{}' and no default backend set",
                    resource_kind
                )
            });

        let provider = match backend_config {
            BackendConfig::Filesystem { path } => {
                let fs_provider = FilesystemDatabaseProvider::<T>::new(path.clone(), 10);
                Arc::new(AnyProvider::Filesystem(fs_provider))
            }
            BackendConfig::Sqlite { .. } => {
                // Here you would instantiate your SqliteDatabaseProvider
                panic!(
                    "Sqlite backend is not implemented yet for kind '{}'",
                    resource_kind
                );
            }
        };

        self.providers.insert(type_id, provider.clone());
        provider
    }

    pub fn ns_provider<T>(&self) -> Arc<AnyNsProvider<T>>
    where
        T: GitopsResourceRoot + Serialize + DeserializeOwned,
    {
        let type_id = TypeId::of::<T>();

        // Fast path: check if the provider is already cached.
        if let Some(entry) = self.providers_ns.get(&type_id) {
            return entry
                .value()
                .clone()
                .downcast::<AnyNsProvider<T>>()
                .unwrap();
        }

        // Slow path: not in cache, so create, insert, and return.
        let resource_kind = T::kind();
        let backend_config = self
            .config
            .resource_backends
            .get(resource_kind)
            .or(self.config.default_backend.as_ref())
            .unwrap_or_else(|| {
                panic!(
                    "No backend configured for kind '{}' and no default backend set",
                    resource_kind
                )
            });

        let provider = match backend_config {
            BackendConfig::Filesystem { path } => {
                let fs_provider = FilesystemNamespacedDatabaseProvider::<T>::new(path.clone(), 10);
                Arc::new(AnyNsProvider::Filesystem(fs_provider))
            }
            BackendConfig::Sqlite { .. } => {
                // Here you would instantiate your SqliteDatabaseProvider
                panic!(
                    "Sqlite backend is not implemented yet for kind '{}'",
                    resource_kind
                );
            }
        };

        self.providers_ns.insert(type_id, provider.clone());
        provider
    }
}

/// A handler that is called after a successful database operation within a transaction.
pub type OnUpdateHandler<T> = Arc<
    dyn Fn(Option<&T>, Option<&T>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + Sync>>
        + Send
        + Sync,
>;

/// A default `OnUpdateHandler` that does nothing.
pub fn default_on_update<T: Send + Sync + 'static>() -> OnUpdateHandler<T> {
    Arc::new(|_before, _after| Box::pin(async { Ok(()) }))
}

/// Represents the state required for a transaction, e.g., for optimistic locking.
#[derive(Clone, Debug)]
pub enum TransactionState {
    File {
        path: PathBuf,
        modified: Option<SystemTime>,
    },
    None,
}

/// A generic database provider for a single type `T`.
pub trait GenericDatabaseProvider<T>: Send + Sync
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned,
{
    async fn list(&self) -> Result<Vec<T>>;
    async fn list_keys(&self) -> Result<Vec<String>>;
    async fn get_by_key(&self, key: &str) -> Result<T>;
    async fn try_get_by_key(&self, key: &str) -> Result<Option<T>>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn insert(&self, item: &T) -> Result<()>;
    async fn upsert(&self, item: &T) -> Result<()>;
}