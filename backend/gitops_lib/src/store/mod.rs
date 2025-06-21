use crate::store::filesystem::{FilesystemDatabaseProvider, FilesystemNamespacedDatabaseProvider, GenericNamespacedDatabaseProvider};
use crate::GitopsResourceRoot;
use anyhow::{anyhow, Context, Result};
use dashmap::DashMap;
use serde::{de::DeserializeOwned, Serialize};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::fs;
use tokio::io;

pub mod config;
pub mod filesystem;
use config::{BackendConfig, StoreConfig};

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

    async fn with_updates<F, Fut, R>(&self, key: &str, retries: u32, update_fn: F) -> Result<R>
    where
        F: Fn(Option<T>) -> Fut + Send + Sync,
        Fut: Future<Output = Result<(Option<T>, R)>> + Send,
        R: Send,
    {
        match self {
            AnyProvider::Filesystem(p) => p.with_updates(key, retries, update_fn).await,
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

    async fn with_updates<F, Fut, R>(
        &self,
        ns: &str,
        key: &str,
        retries: u32,
        update_fn: F,
    ) -> Result<R>
    where
        F: Fn(Option<T>) -> Fut + Send + Sync,
        Fut: Future<Output = Result<(Option<T>, R)>> + Send,
        R: Send,
    {
        match self {
            AnyNsProvider::Filesystem(p) => p.with_updates(ns, key, retries, update_fn).await,
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
                let fs_provider = FilesystemDatabaseProvider::<T>::new(path.clone(), None);
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
                let fs_provider =
                    FilesystemNamespacedDatabaseProvider::<T>::new(path.clone(), None);
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

/// Error type for optimistic locking failures.
#[derive(thiserror::Error, Debug)]
#[error("Optimistic lock failed: resource was modified by another process.")]
pub struct OptimisticLockError;

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
    async fn with_updates<F, Fut, R>(&self, key: &str, retries: u32, update_fn: F) -> Result<R>
    where
        F: Fn(Option<T>) -> Fut + Send + Sync,
        Fut: Future<Output = Result<(Option<T>, R)>> + Send,
        R: Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GitopsEnum, GitopsResourcePart, GitopsResourceRoot};
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    // --- Test Resource Definitions ---

    #[derive(GitopsResourceRoot, Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
    #[gitops(key = "name", api_version = "example.com/v1", kind = "User")]
    pub struct User {
        pub name: String,
        pub email: Option<String>,
    }

    #[derive(GitopsResourceRoot, Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
    #[gitops(key = "name", api_version = "example.com/v1", kind = "Project")]
    pub struct Project {
        pub name: String,
        pub active: bool,
    }

    #[tokio::test]
    async fn test_dynamic_backend_selection_with_store() {
        let users_dir = tempdir().unwrap();
        let projects_dir = tempdir().unwrap();

        // 1. Define a configuration that maps resource kinds to different backends.
        let config = StoreConfig {
            default_backend: None, // No default, forcing explicit configuration
            resource_backends: HashMap::from([
                (
                    "User".to_string(),
                    BackendConfig::Filesystem {
                        path: users_dir.path().to_path_buf(),
                    },
                ),
                (
                    "Project".to_string(),
                    BackendConfig::Filesystem {
                        path: projects_dir.path().to_path_buf(),
                    },
                ),
            ]),
            namespaced_resource_backends: HashMap::new(),
            namespace_backends: HashMap::new(),
        };

        // 2. Create the Store, which acts as our application's central data access layer.
        let store = Store::new(config);

        // 3. In a handler, get a provider for the `User` type.
        // The Store will see that "User" maps to a filesystem backend at `users_dir`.
        let user_provider = store.provider::<User>();

        // Verify it's the correct type (for testing purposes)
        assert!(matches!(&*user_provider, AnyProvider::Filesystem(_)));

        // 4. Use the provider to work with Users.
        let user1 = User {
            name: "alice".into(),
            email: Some("alice@example.com".into()),
        };
        user_provider.insert(&user1).await.unwrap();
        let fetched_user = user_provider.get_by_key("alice").await.unwrap();
        assert_eq!(user1, fetched_user);

        // 5. Get a provider for the `Project` type.
        // The Store will see "Project" maps to a *different* filesystem backend at `projects_dir`.
        let project_provider = store.provider::<Project>();

        // 6. Use the provider to work with Projects.
        let proj1 = Project {
            name: "secret-project".into(),
            active: true,
        };
        project_provider.insert(&proj1).await.unwrap();
        let fetched_project = project_provider.get_by_key("secret-project").await.unwrap();
        assert_eq!(proj1, fetched_project);

        // 7. Crucially, verify that the data is isolated because the providers point to different directories.
        assert!(user_provider.get_by_key("secret-project").await.is_err());
        assert!(project_provider.get_by_key("alice").await.is_err());

        // Check the actual filesystem to be sure
        let user_file_path = users_dir.path().join("User").join("alice.yaml");
        assert!(user_file_path.exists());
        let project_file_path = projects_dir
            .path()
            .join("Project")
            .join("secret-project.yaml");
        assert!(project_file_path.exists());
        let non_existent_path = users_dir.path().join("Project");
        assert!(!non_existent_path.exists());
    }
}
