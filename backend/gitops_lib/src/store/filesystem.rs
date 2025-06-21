use crate::store::{default_on_update, GenericDatabaseProvider, OnUpdateHandler, OptimisticLockError, TransactionState};
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
            if path.exists() {
                let current_meta = fs::metadata(&path).await?;
                if current_meta.modified()? != *modified {
                    return Err(OptimisticLockError.into());
                }
            } else {
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
                    match fs::remove_file(&path).await {
                        Ok(_) => {}
                        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                        Err(e) => return Err(e.into()),
                    }
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
        let item = item.clone();
        self.with_updates(&key, 0, |existing| {
            let key = key.clone();
            let item = item.clone();
            async move {
                if existing.is_some() {
                    Err(anyhow!("Resource with key '{}' already exists", key))
                } else {
                    Ok((Some(item), ()))
                }
            }
        })
        .await
    }

    async fn upsert(&self, item: &T) -> Result<()> {
        let key = item.get_key();
        let item = item.clone();
        self.with_updates(&key, 0, |_| {
            let item = item.clone();
            async move { Ok((Some(item), ())) }
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

// Namespaced provider implementation remains unchanged for now, but could be integrated
// with the new Store in a similar fashion if needed.
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
        self.get_type_path()
            .join(T::kind())
            .join(urlencoding::encode(ns).as_ref())
    }

    fn get_type_path(&self) -> PathBuf {
        self.base_path.join(T::kind())
    }

    fn get_resource_path(&self, ns: &str, key: &str) -> PathBuf {
        self.get_ns_path(ns)
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
