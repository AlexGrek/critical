use std::{collections::{HashMap, HashSet}, path::Path, sync::RwLock};

use sled::Tree;

use crate::store::{
    StorageError,
    qstorage::{KvStorage, StorageResult},
};

/// Sled-backed implementation of KvStorage.
pub struct SledKv {
    db: sled::Db,
    trees: RwLock<HashMap<String, Tree>>, // RwLock for concurrent reads and writes
}

impl SledKv {
    pub fn new<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        let db = sled::open(path).map_err(|e| StorageError::StorageError {
            reason: format!("Failed to open sled DB: {e}"),
        })?;
        Ok(Self {
            db,
            trees: RwLock::new(HashMap::new()),
        })
    }

    fn get_tree(&self, store: &str) -> StorageResult<Tree> {
        {
            let read_guard = self.trees.read().unwrap();
            if let Some(tree) = read_guard.get(store) {
                return Ok(tree.clone());
            }
        }

        let tree = self
            .db
            .open_tree(store)
            .map_err(|e| StorageError::StorageError {
                reason: format!("Failed to open sled tree: {e}"),
            })?;

        let mut write_guard = self.trees.write().unwrap();
        write_guard.insert(store.to_string(), tree.clone());

        Ok(tree)
    }
}

impl KvStorage for SledKv {
    fn initialize(&mut self, store: &str) -> StorageResult<()> {
        let tree = self
            .db
            .open_tree(store)
            .map_err(|e| StorageError::StorageError {
                reason: format!("Failed to initialize sled tree: {e}"),
            })?;
        self.trees.write().unwrap().insert(store.to_string(), tree);
        Ok(())
    }

    fn get(&self, store: &str, key: &str) -> StorageResult<Vec<String>> {
        let tree = self.get_tree(store)?;
        let bytes = tree
            .get(key.as_bytes())
            .map_err(|e| StorageError::StorageError {
                reason: format!("Sled get error: {e}"),
            })?
            .ok_or_else(|| StorageError::ItemNotFound {
                key: key.to_string(),
                kind: store.to_string(),
            })?;

        let (parsed, _len) = bincode::serde::decode_from_slice::<
            Vec<String>,
            bincode::config::Configuration,
        >(&bytes, bincode::config::standard())
        .map_err(|e| StorageError::ReadItemFailure {
            reason: format!("{e}"),
        })?;

        Ok(parsed)
    }

    fn set(&mut self, store: &str, key: &str, value: Vec<String>) -> StorageResult<()> {
        let tree = self.get_tree(store)?;
        let serialized =
            bincode::encode_to_vec(&value, bincode::config::standard()).map_err(|e| {
                StorageError::WriteItemFailure {
                    reason: format!("{e}"),
                }
            })?;

        tree.insert(key.as_bytes(), serialized)
            .map_err(|e| StorageError::StorageError {
                reason: format!("Sled insert error: {e}"),
            })?;

        Ok(())
    }
}
