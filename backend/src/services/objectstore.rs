use std::sync::Arc;

use bytes::Bytes;
use futures_util::StreamExt;
use object_store::{ObjectMeta, ObjectStore, path::Path};

use crate::config::AppConfig;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("object store error: {0}")]
    Store(#[from] object_store::Error),
    #[error("invalid path: {0}")]
    Path(#[from] object_store::path::Error),
    #[error("object store not configured")]
    NotConfigured,
    #[error("unsupported backend: {0}")]
    UnsupportedBackend(String),
}

pub struct ObjectStoreService {
    store: Arc<dyn ObjectStore>,
}

impl ObjectStoreService {
    pub fn new(config: &AppConfig) -> Result<Self, StorageError> {
        let store: Arc<dyn ObjectStore> = match config.object_store_backend.as_str() {
            "local" => {
                use object_store::local::LocalFileSystem;
                let fs = LocalFileSystem::new_with_prefix(&config.object_store_path)?;
                Arc::new(fs)
            }
            "s3" => {
                use object_store::aws::AmazonS3Builder;
                let mut builder = AmazonS3Builder::new()
                    .with_bucket_name(&config.object_store_bucket)
                    .with_region(&config.object_store_region)
                    .with_access_key_id(&config.object_store_key)
                    .with_secret_access_key(&config.object_store_secret);
                if !config.object_store_url.is_empty() {
                    builder = builder.with_endpoint(&config.object_store_url);
                }
                Arc::new(builder.build()?)
            }
            "webdav" => {
                use object_store::http::HttpBuilder;
                let store = HttpBuilder::new()
                    .with_url(&config.object_store_url)
                    .build()?;
                Arc::new(store)
            }
            other => return Err(StorageError::UnsupportedBackend(other.to_string())),
        };

        Ok(Self { store })
    }

    /// Tries to construct the service from config. Returns `None` (with a warning) if
    /// `OBJECT_STORE_BACKEND` is not set or initialization fails.
    pub fn try_from_config(config: &AppConfig) -> Option<Self> {
        if config.object_store_backend.is_empty() {
            log::info!("[objectstore] OBJECT_STORE_BACKEND not set — running without object store");
            return None;
        }
        match Self::new(config) {
            Ok(svc) => {
                log::info!("[objectstore] initialized backend: {}", config.object_store_backend);
                Some(svc)
            }
            Err(e) => {
                log::warn!("[objectstore] failed to initialize: {e} — running without object store");
                None
            }
        }
    }

    pub async fn put(&self, path: &str, data: Bytes) -> Result<(), StorageError> {
        let location = Path::parse(path)?;
        self.store.put(&location, data.into()).await?;
        Ok(())
    }

    pub async fn get(&self, path: &str) -> Result<Bytes, StorageError> {
        let location = Path::parse(path)?;
        let result = self.store.get(&location).await?;
        Ok(result.bytes().await?)
    }

    pub async fn delete(&self, path: &str) -> Result<(), StorageError> {
        let location = Path::parse(path)?;
        self.store.delete(&location).await?;
        Ok(())
    }

    pub async fn list(&self, prefix: &str) -> Result<Vec<ObjectMeta>, StorageError> {
        let prefix_path = if prefix.is_empty() {
            None
        } else {
            Some(Path::parse(prefix)?)
        };
        let mut stream = self.store.list(prefix_path.as_ref());
        let mut results = Vec::new();
        while let Some(meta) = stream.next().await {
            results.push(meta?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use object_store::memory::InMemory;

    fn memory_service() -> ObjectStoreService {
        ObjectStoreService {
            store: Arc::new(InMemory::new()),
        }
    }

    #[tokio::test]
    async fn test_put_get_delete() {
        let svc = memory_service();
        let data = Bytes::from("hello object store");

        svc.put("test/hello.txt", data.clone()).await.unwrap();
        let got = svc.get("test/hello.txt").await.unwrap();
        assert_eq!(got, data);

        svc.delete("test/hello.txt").await.unwrap();
        assert!(svc.get("test/hello.txt").await.is_err());
    }

    #[tokio::test]
    async fn test_list_with_prefix() {
        let svc = memory_service();
        svc.put("docs/a.txt", Bytes::from("a")).await.unwrap();
        svc.put("docs/b.txt", Bytes::from("b")).await.unwrap();
        svc.put("other/c.txt", Bytes::from("c")).await.unwrap();

        let results = svc.list("docs").await.unwrap();
        assert_eq!(results.len(), 2);
    }
}
