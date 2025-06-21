// In your main lib or a testing module
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use thiserror::Error;

use crate::db::core::DatabaseProvider;

#[derive(Error, Debug)]
pub enum MockDbError {
    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Transaction not found or invalid")]
    InvalidTransaction,
}

// Our transaction is just a copy of the database state at the start.
// A real DB would use a more sophisticated mechanism (e.g., a connection handle).
pub type MockTransaction = HashMap<String, String>;

#[derive(Clone, Default)]
pub struct MockDb {
    store: Arc<Mutex<HashMap<String, String>>>,
}

impl MockDb {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl DatabaseProvider for MockDb {
    type Error = MockDbError;
    type Transaction = MockTransaction;

    async fn start_transaction(&self) -> Result<Self::Transaction, Self::Error> {
        // Start of transaction: clone the current state.
        Ok(self.store.lock().unwrap().clone())
    }

    async fn commit_transaction(&self, tx: Self::Transaction) -> Result<(), Self::Error> {
        // On commit, replace the main store with the transaction's state.
        let mut store = self.store.lock().unwrap();
        *store = tx;
        Ok(())
    }
    
    async fn rollback_transaction(&self, _tx: Self::Transaction) -> Result<(), Self::Error> {
        // On rollback, we do nothing, abandoning the transaction state.
        Ok(())
    }

    async fn get<T: DeserializeOwned + Send>(
        &self,
        key: &str,
        tx: Option<&mut Self::Transaction>,
    ) -> Result<Option<T>, Self::Error> {
        let map = match tx {
            Some(t) => t, // Use transaction state if provided
            None => &*self.store.lock().unwrap(),
        };
        map.get(key)
            .map(|v| serde_json::from_str(v))
            .transpose()
            .map_err(Into::into)
    }

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        tx: &mut Self::Transaction,
    ) -> Result<(), Self::Error> {
        let value_str = serde_json::to_string(value)?;
        // All mutations happen on the transaction state
        tx.insert(key.to_string(), value_str);
        Ok(())
    }

    async fn delete(&self, key: &str, tx: &mut Self::Transaction) -> Result<Option<String>, Self::Error> {
        // All mutations happen on the transaction state
        Ok(tx.remove(key))
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>, Self::Error> {
        let store = self.store.lock().unwrap();
        Ok(store.keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect())
    }
}