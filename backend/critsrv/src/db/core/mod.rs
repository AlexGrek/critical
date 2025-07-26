// In your traits file
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::error::Error;

pub mod tests;

#[async_trait]
pub trait DatabaseProvider {
    type Error: Error + Send + Sync;
    type Transaction; // Represents the state of a transaction

    // Transaction Management
    async fn start_transaction(&self) -> Result<Self::Transaction, Self::Error>;
    async fn commit_transaction(&self, tx: Self::Transaction) -> Result<(), Self::Error>;
    async fn rollback_transaction(&self, tx: Self::Transaction) -> Result<(), Self::Error>;

    // Key-Value Operations (now accept an optional transaction)
    async fn get_resource<T: DeserializeOwned + Send + Sync>(
        &self,
        resource_type: &str,
        key: &str,
        tx: Option<&mut Self::Transaction>,
    ) -> Result<Option<T>, Self::Error>;

    async fn get_resource_ns<T: DeserializeOwned + Send + Sync>(
        &self,
        resource_type: &str,
        namespace: &str,
        key: &str,
        tx: Option<&mut Self::Transaction>,
    ) -> Result<Option<T>, Self::Error>;

    async fn set_resource<T: Serialize + Send + Sync>(
        &self,
        resource_type: &str,
        key: &str,
        value: &T,
        tx: &mut Self::Transaction,
    ) -> Result<(), Self::Error>;

    async fn set_resource_ns<T: Serialize + Send + Sync>(
        &self,
        resource_type: &str,
        namespace: &str,
        key: &str,
        value: &T,
        tx: &mut Self::Transaction,
    ) -> Result<(), Self::Error>;

    async fn delete_resource(
        &self,
        resource_type: &str,
        key: &str,
        tx: &mut Self::Transaction,
    ) -> Result<Option<String>, Self::Error>;

    async fn delete_resource_ns(
        &self,
        resource_type: &str,
        namespace: &str,
        key: &str,
        tx: &mut Self::Transaction,
    ) -> Result<Option<String>, Self::Error>;

    async fn list_keys(&self, resource_type: &str) -> Result<Vec<String>, Self::Error>;

    async fn list_keys_ns(&self, resource_type: &str, namespace: &str) -> Result<Vec<String>, Self::Error>;
}
