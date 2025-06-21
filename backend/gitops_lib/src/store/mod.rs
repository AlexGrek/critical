use crate::GitopsResourceRoot;
use anyhow::{anyhow, Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::fs;
use tokio::io;

/// A handler that is called after a successful database operation within a transaction.
///
/// It receives the state of the object before the operation (`before`) and after (`after`).
/// - Create: `before` is `None`, `after` is `Some`.
/// - Delete: `before` is `Some`, `after` is `None`.
/// - Update: `before` is `Some`, `after` is `Some`.
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
    // Other states for different DBs can be added here.
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
    /// Lists all resources of type `T`.
    async fn list(&self) -> Result<Vec<T>>;

    /// Lists all resource keys of type `T`.
    async fn list_keys(&self) -> Result<Vec<String>>;

    /// Retrieves a resource by its key. Returns an error if not found.
    async fn get_by_key(&self, key: &str) -> Result<T>;

    /// Tries to retrieve a resource by its key. Returns `Ok(None)` if not found.
    async fn try_get_by_key(&self, key: &str) -> Result<Option<T>>;

    /// Deletes a resource by its key.
    async fn delete(&self, key: &str) -> Result<()>;

    /// Inserts a new resource. Fails if a resource with the same key already exists.
    async fn insert(&self, item: &T) -> Result<()>;

    /// Inserts a new resource or updates an existing one.
    async fn upsert(&self, item: &T) -> Result<()>;

    /// Performs a transactional read-modify-write operation on a resource.
    ///
    /// It will retry the operation up to `retries` times if an `OptimisticLockError` occurs.
    /// The `update_fn` receives the current state of the resource (`None` if it doesn't exist)
    /// and should return the desired new state (`None` to delete) and a result `R`.
    async fn with_updates<F, Fut, R>(&self, key: &str, retries: u32, update_fn: F) -> Result<R>
    where
        F: Fn(Option<T>) -> Fut + Send + Sync,
        Fut: Future<Output = Result<(Option<T>, R)>> + Send,
        R: Send;
}

/// A filesystem-based implementation of `GenericDatabaseProvider`.
pub struct FilesystemDatabaseProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned,
{
    base_path: PathBuf,
    on_update: OnUpdateHandler<T>,
    _phantom: PhantomData<T>,
}

impl<T> FilesystemDatabaseProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned,
{
    /// Creates a new `FilesystemDatabaseProvider`.
    ///
    /// # Arguments
    ///
    /// * `base_path` - The root directory for storing data.
    /// * `on_update` - An optional event handler to be called on data changes.
    pub fn new(base_path: impl Into<PathBuf>, on_update: Option<OnUpdateHandler<T>>) -> Self {
        Self {
            base_path: base_path.into(),
            on_update: on_update.unwrap_or_else(default_on_update),
            _phantom: PhantomData,
        }
    }

    fn get_resource_path(&self, key: &str) -> PathBuf {
        self.base_path
            .join(T::kind())
            .join(format!("{}.yaml", urlencoding::encode(key)))
    }

    fn get_type_path(&self) -> PathBuf {
        self.base_path.join(T::kind())
    }

    async fn get_with_transaction_state(&self, key: &str) -> Result<(Option<T>, TransactionState)> {
        let path = self.get_resource_path(key);
        match fs::read_to_string(&path).await {
            Ok(content) => {
                let meta = fs::metadata(&path).await?;
                let resource: T::Serializable = serde_yaml::from_str(&content)?;
                Ok((
                    Some(T::from(resource)),
                    TransactionState::File {
                        path,
                        modified: Some(meta.modified()?),
                    },
                ))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok((
                None,
                TransactionState::File {
                    path,
                    modified: None,
                },
            )),
            Err(e) => Err(e.into()),
        }
    }

    async fn write_with_transaction_state(
        &self,
        new_item: Option<&T>,
        state: &TransactionState,
    ) -> Result<()> {
        let (path, expected_modified_time) = match state {
            TransactionState::File { path, modified } => (path, modified),
            _ => return Err(anyhow!("Invalid transaction state for filesystem DB")),
        };

        if let Some(modified) = expected_modified_time {
            let current_meta = fs::metadata(&path).await?;
            if current_meta.modified()? != *modified {
                return Err(OptimisticLockError.into());
            }
        }

        match new_item {
            Some(item) => {
                let serializable_item = item.as_serializable();
                let yaml_content = serde_yaml::to_string(&serializable_item)?;
                let parent_dir = path.parent().ok_or_else(|| {
                    anyhow!("Failed to get parent directory for path: {:?}", path)
                })?;
                fs::create_dir_all(parent_dir).await?;
                fs::write(&path, yaml_content).await?;
            }
            None => {
                if expected_modified_time.is_some() {
                    fs::remove_file(&path).await?;
                }
            }
        }
        Ok(())
    }
}

impl<T> GenericDatabaseProvider<T> for FilesystemDatabaseProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned,
{
    async fn list(&self) -> Result<Vec<T>> {
        let dir_path = self.get_type_path();
        let mut resources = Vec::new();

        if !dir_path.exists() {
            return Ok(resources);
        }

        let mut entries = fs::read_dir(&dir_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml") {
                let content = fs::read_to_string(&path).await?;
                let resource: T::Serializable = serde_yaml::from_str(&content)?;
                resources.push(resource.into());
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

        let mut entries = fs::read_dir(&dir_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml") {
                if let Some(stem) = path.file_stem() {
                    if let Some(key_str) = stem.to_str() {
                        keys.push(urlencoding::decode(key_str)?.into_owned());
                    }
                }
            }
        }
        Ok(keys)
    }

    async fn get_by_key(&self, key: &str) -> Result<T> {
        self.try_get_by_key(key)
            .await?
            .ok_or_else(|| anyhow!("Resource with key '{}' not found", key))
    }

    async fn try_get_by_key(&self, key: &str) -> Result<Option<T>> {
        self.get_with_transaction_state(key)
            .await
            .map(|(item, _)| item)
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.with_updates(key, 0, |_| async { Ok((None, ())) })
            .await
    }

    async fn insert(&self, item: &T) -> Result<()> {
        let key = item.get_key();
        let item_clone = item.clone();
        self.with_updates(&key, 0, |existing| {
            let key = key.clone();
            let item_clone = item_clone.clone();
            async move {
                if existing.is_some() {
                    Err(anyhow!("Resource with key '{}' already exists", key))
                } else {
                    Ok((Some(item_clone), ()))
                }
            }
        })
        .await
    }

    async fn upsert(&self, item: &T) -> Result<()> {
        let key = item.get_key();
        let item_clone = item.clone();
        self.with_updates(&key, 0, |_| {
            let item_clone = item_clone.clone();
            async move { Ok((Some(item_clone), ())) }
        })
        .await
    }

    async fn with_updates<F, Fut, R>(&self, key: &str, retries: u32, update_fn: F) -> Result<R>
    where
        F: Fn(Option<T>) -> Fut + Send + Sync,
        Fut: Future<Output = Result<(Option<T>, R)>> + Send,
        R: Send,
    {
        let mut attempts = 0;
        loop {
            let (before, tx_state) = self.get_with_transaction_state(key).await?;

            let (after, result) = update_fn(before.clone()).await?;

            let write_result = self
                .write_with_transaction_state(after.as_ref(), &tx_state)
                .await;

            match write_result {
                Ok(()) => {
                    (self.on_update)(before.as_ref(), after.as_ref()).await?;
                    return Ok(result);
                }
                Err(e) => {
                    if e.downcast_ref::<OptimisticLockError>().is_some() {
                        attempts += 1;
                        if attempts > retries {
                            return Err(e).context(format!(
                                "Optimistic lock failed after {} retries",
                                retries
                            ));
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            50 * attempts as u64,
                        ))
                        .await;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }
}

/// A generic database provider that supports namespaces.
pub trait GenericNamespacedDatabaseProvider<T>: Send + Sync
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned,
{
    async fn list(&self, ns: &str) -> Result<Vec<T>>;
    async fn list_keys(&self, ns: &str) -> Result<Vec<String>>;
    async fn get_by_key(&self, ns: &str, key: &str) -> Result<T>;
    async fn try_get_by_key(&self, ns: &str, key: &str) -> Result<Option<T>>;
    async fn delete(&self, ns: &str, key: &str) -> Result<()>;
    async fn insert(&self, ns: &str, item: &T) -> Result<()>;
    async fn upsert(&self, ns: &str, item: &T) -> Result<()>;

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
        R: Send;

    async fn list_namespaces(&self) -> Result<Vec<String>>;
    async fn create_namespace(&self, ns: &str) -> Result<()>;
    async fn delete_namespace(&self, ns: &str, force: bool) -> Result<()>;
}

/// A filesystem-based implementation of `GenericNamespacedDatabaseProvider`.
pub struct FilesystemNamespacedDatabaseProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned,
{
    base_path: PathBuf,
    on_update: OnUpdateHandler<T>,
    _phantom: PhantomData<T>,
}

impl<T> FilesystemNamespacedDatabaseProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned,
{
    pub fn new(base_path: impl Into<PathBuf>, on_update: Option<OnUpdateHandler<T>>) -> Self {
        Self {
            base_path: base_path.into(),
            on_update: on_update.unwrap_or_else(default_on_update),
            _phantom: PhantomData,
        }
    }

    fn get_ns_path(&self, ns: &str) -> PathBuf {
        self.base_path.join(urlencoding::encode(ns).as_ref())
    }

    fn get_type_path(&self, ns: &str) -> PathBuf {
        self.get_ns_path(ns).join(T::kind())
    }

    fn get_resource_path(&self, ns: &str, key: &str) -> PathBuf {
        self.get_type_path(ns)
            .join(format!("{}.yaml", urlencoding::encode(key)))
    }

    fn provider_for_namespace(&self, ns: &str) -> FilesystemDatabaseProvider<T> {
        FilesystemDatabaseProvider::new(self.get_ns_path(ns), Some(self.on_update.clone()))
    }
}

impl<T> GenericNamespacedDatabaseProvider<T> for FilesystemNamespacedDatabaseProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned,
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
            fs::create_dir_all(&ns_path).await?;
        }
        self.provider_for_namespace(ns).upsert(item).await
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
        self.provider_for_namespace(ns)
            .with_updates(key, retries, update_fn)
            .await
    }

    async fn list_namespaces(&self) -> Result<Vec<String>> {
        let mut namespaces = Vec::new();
        if !self.base_path.exists() {
            return Ok(namespaces);
        }
        let mut entries = fs::read_dir(&self.base_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    namespaces.push(urlencoding::decode(name)?.into_owned());
                }
            }
        }
        Ok(namespaces)
    }

    async fn create_namespace(&self, ns: &str) -> Result<()> {
        fs::create_dir_all(self.get_ns_path(ns))
            .await
            .map_err(Into::into)
    }

    async fn delete_namespace(&self, ns: &str, force: bool) -> Result<()> {
        let ns_path = self.get_ns_path(ns);
        if !ns_path.exists() {
            return Ok(());
        }

        if !force {
            let mut entries = fs::read_dir(&ns_path).await?;
            if entries.next_entry().await?.is_some() {
                return Err(anyhow!(
                    "Cannot delete non-empty namespace '{}' without 'force=true'",
                    ns
                ));
            }
        }

        fs::remove_dir_all(&ns_path).await.map_err(Into::into)
    }
}

// Cloning a FilesystemDatabaseProvider should be possible for use across threads.
impl<T> Clone for FilesystemDatabaseProvider<T>
where
    T: GitopsResourceRoot + Serialize + DeserializeOwned,
{
    fn clone(&self) -> Self {
        Self {
            base_path: self.base_path.clone(),
            on_update: self.on_update.clone(),
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
    T: GitopsResourceRoot + Serialize + DeserializeOwned,
{
    fn clone(&self) -> Self {
        Self {
            base_path: self.base_path.clone(),
            on_update: self.on_update.clone(),
            _phantom: PhantomData,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GitopsEnum, GitopsResourcePart, GitopsResourceRoot};
    use serde::{Deserialize, Serialize};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::tempdir;

    // --- Test Resource Definitions ---

    #[derive(GitopsResourcePart, Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub struct Status {
        pub ready_replicas: u32,
        pub available_replicas: u32,
        pub conditions: Vec<String>,
    }

    #[derive(GitopsEnum, Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub enum UserStatus {
        Fired,
        Replaced,
        Normal,
    }

    /// The root GitOps resource for a Deployment.
    #[derive(GitopsResourceRoot, Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
    #[gitops(key = "name", api_version = "apps.example.com/v1")]
    pub struct Deployment {
        pub name: String,
        #[gitops(skip_on_update)]
        pub creation_timestamp: String,
        pub status: Option<UserStatus>,
        pub additional_info: Option<String>,
    }

    // --- Tests for FilesystemDatabaseProvider ---

    #[tokio::test]
    async fn test_fs_provider_insert_and_get() {
        let dir = tempdir().unwrap();
        let db = FilesystemDatabaseProvider::<Deployment>::new(dir.path(), None);
        let deployment = Deployment {
            name: "test-app".to_string(),
            creation_timestamp: "now".to_string(),
            status: Some(UserStatus::Normal),
            additional_info: None,
        };

        db.insert(&deployment).await.unwrap();
        let fetched = db.get_by_key("test-app").await.unwrap();
        assert_eq!(deployment, fetched);

        // Inserting again should fail
        assert!(db.insert(&deployment).await.is_err());
    }

    #[tokio::test]
    async fn test_fs_provider_upsert() {
        let dir = tempdir().unwrap();
        let db = FilesystemDatabaseProvider::<Deployment>::new(dir.path(), None);
        let mut deployment = Deployment {
            name: "test-app".to_string(),
            creation_timestamp: "now".to_string(),
            status: Some(UserStatus::Normal),
            additional_info: Some("initial".to_string()),
        };

        // Upsert should create
        db.upsert(&deployment).await.unwrap();
        let fetched = db.get_by_key("test-app").await.unwrap();
        assert_eq!(fetched.additional_info, Some("initial".to_string()));

        // Upsert should update
        deployment.additional_info = Some("updated".to_string());
        db.upsert(&deployment).await.unwrap();
        let fetched_updated = db.get_by_key("test-app").await.unwrap();
        assert_eq!(fetched_updated.additional_info, Some("updated".to_string()));
    }

    #[tokio::test]
    async fn test_fs_provider_delete() {
        let dir = tempdir().unwrap();
        let db = FilesystemDatabaseProvider::<Deployment>::new(dir.path(), None);
        let deployment = Deployment {
            name: "to-delete".to_string(),
            creation_timestamp: "tbd".to_string(),
            status: None,
            additional_info: None,
        };

        db.insert(&deployment).await.unwrap();
        assert!(db.get_by_key("to-delete").await.is_ok());

        db.delete("to-delete").await.unwrap();
        assert!(db.get_by_key("to-delete").await.is_err());
    }

    #[tokio::test]
    async fn test_fs_provider_list() {
        let dir = tempdir().unwrap();
        let db = FilesystemDatabaseProvider::<Deployment>::new(dir.path(), None);

        db.insert(&Deployment {
            name: "app1".into(),
            ..Default::default()
        })
        .await
        .unwrap();
        db.insert(&Deployment {
            name: "app2".into(),
            ..Default::default()
        })
        .await
        .unwrap();

        let keys = db.list_keys().await.unwrap();
        let mut sorted_keys = keys;
        sorted_keys.sort();
        assert_eq!(sorted_keys.len(), 2);
        assert!(sorted_keys.contains(&"app1".to_string()));

        let items = db.list().await.unwrap();
        assert_eq!(items.len(), 2);
    }

    #[tokio::test]
    async fn test_fs_provider_with_updates_and_retries() {
        let dir = tempdir().unwrap();
        let db = FilesystemDatabaseProvider::<Deployment>::new(dir.path(), None);
        let deployment = Deployment {
            name: "tx-app".into(),
            ..Default::default()
        };
        db.insert(&deployment).await.unwrap();

        // Simulate a concurrent modification
        let db_clone = FilesystemDatabaseProvider::<Deployment>::new(dir.path(), None);

        let attempts = Arc::new(AtomicUsize::new(0));

        let update_task = tokio::spawn({
            let db = db.clone();
            let attempts = attempts.clone();
            async move {
                db.with_updates("tx-app", 5, |maybe_item| {
                    let attempts = attempts.clone();
                    let db_clone = db_clone.clone();
                    async move {
                        attempts.fetch_add(1, Ordering::SeqCst);
                        let mut item = maybe_item.unwrap();

                        if attempts.load(Ordering::SeqCst) == 1 {
                            // On the first attempt, we make an external modification
                            // to cause an optimistic lock failure.
                            let mut conflicting_item = item.clone();
                            conflicting_item.additional_info = Some("conflict".to_string());
                            db_clone.upsert(&conflicting_item).await.unwrap();
                        }

                        item.additional_info = Some("success".to_string());
                        Ok((Some(item), "done".to_string()))
                    }
                })
                .await
            }
        });

        let result = update_task.await.unwrap().unwrap();
        assert_eq!(result, "done");
        assert_eq!(attempts.load(Ordering::SeqCst), 2); // Should succeed on the 2nd try

        let final_item = FilesystemDatabaseProvider::<Deployment>::new(dir.path(), None)
            .get_by_key("tx-app")
            .await
            .unwrap();
        assert_eq!(final_item.additional_info, Some("success".to_string()));
    }

    #[tokio::test]
    async fn test_fs_provider_on_update_handler() {
        let dir = tempdir().unwrap();
        let creation_count = Arc::new(AtomicUsize::new(0));
        let update_count = Arc::new(AtomicUsize::new(0));
        let deletion_count = Arc::new(AtomicUsize::new(0));

        let cc_clone = creation_count.clone();
        let uc_clone = update_count.clone();
        let dc_clone = deletion_count.clone();

        let on_update: OnUpdateHandler<Deployment> = Arc::new(move |before, after| {
            let cc = cc_clone.clone();
            let uc = uc_clone.clone();
            let dc = dc_clone.clone();
            Box::pin(async move {
                match (before, after) {
                    (None, Some(_)) => {
                        cc.fetch_add(1, Ordering::SeqCst);
                    }
                    (Some(_), Some(_)) => {
                        uc.fetch_add(1, Ordering::SeqCst);
                    }
                    (Some(_), None) => {
                        dc.fetch_add(1, Ordering::SeqCst);
                    }
                    _ => {}
                }
                Ok(())
            })
        });

        let db = FilesystemDatabaseProvider::<Deployment>::new(dir.path(), Some(on_update));

        // Create
        let deployment = Deployment {
            name: "handler-test".into(),
            ..Default::default()
        };
        db.insert(&deployment).await.unwrap();
        assert_eq!(creation_count.load(Ordering::SeqCst), 1);

        // Update
        let mut updated_deployment = deployment.clone();
        updated_deployment.additional_info = Some("info".to_string());
        db.upsert(&updated_deployment).await.unwrap();
        assert_eq!(update_count.load(Ordering::SeqCst), 1);

        // Delete
        db.delete("handler-test").await.unwrap();
        assert_eq!(deletion_count.load(Ordering::SeqCst), 1);
    }

    // --- Tests for FilesystemNamespacedDatabaseProvider ---

    #[tokio::test]
    async fn test_fs_namespaced_provider_crud() {
        let dir = tempdir().unwrap();
        let db = FilesystemNamespacedDatabaseProvider::<Deployment>::new(dir.path(), None);
        let deployment = Deployment {
            name: "ns-app".into(),
            ..Default::default()
        };

        // Insert into "ns1"
        db.insert("ns1", &deployment).await.unwrap();
        let fetched = db.get_by_key("ns1", "ns-app").await.unwrap();
        assert_eq!(deployment, fetched);

        // Should not exist in "ns2"
        assert!(db.get_by_key("ns2", "ns-app").await.is_err());

        // List keys
        let keys_ns1 = db.list_keys("ns1").await.unwrap();
        assert_eq!(keys_ns1, vec!["ns-app"]);
        let keys_ns2 = db.list_keys("ns2").await.unwrap();
        assert!(keys_ns2.is_empty());

        // Delete
        db.delete("ns1", "ns-app").await.unwrap();
        assert!(db.get_by_key("ns1", "ns-app").await.is_err());
    }

    #[tokio::test]
    async fn test_fs_namespaced_provider_namespace_management() {
        let dir = tempdir().unwrap();
        let db = FilesystemNamespacedDatabaseProvider::<Deployment>::new(dir.path(), None);

        // Create
        db.create_namespace("ns-a").await.unwrap();
        db.create_namespace("ns-b").await.unwrap();

        let mut namespaces = db.list_namespaces().await.unwrap();
        namespaces.sort();
        assert_eq!(namespaces, vec!["ns-a", "ns-b"]);

        // Upsert implicitly creates namespace
        let deployment = Deployment {
            name: "auto-ns".into(),
            ..Default::default()
        };
        db.upsert("ns-c", &deployment).await.unwrap();
        let mut namespaces = db.list_namespaces().await.unwrap();
        namespaces.sort();
        assert_eq!(namespaces, vec!["ns-a", "ns-b", "ns-c"]);

        // Delete empty ns
        db.delete_namespace("ns-a", false).await.unwrap();

        // Fail to delete non-empty ns without force
        assert!(db.delete_namespace("ns-c", false).await.is_err());

        // Succeed to delete non-empty ns with force
        db.delete_namespace("ns-c", true).await.unwrap();

        let mut namespaces_after_delete = db.list_namespaces().await.unwrap();
        namespaces_after_delete.sort();
        assert_eq!(namespaces_after_delete, vec!["ns-b"]);
    }

    // Cloning a FilesystemDatabaseProvider should be possible for use across threads.
    impl<T> Clone for FilesystemDatabaseProvider<T>
    where
        T: GitopsResourceRoot + Serialize + DeserializeOwned,
    {
        fn clone(&self) -> Self {
            Self {
                base_path: self.base_path.clone(),
                on_update: self.on_update.clone(),
                _phantom: PhantomData,
            }
        }
    }
}
