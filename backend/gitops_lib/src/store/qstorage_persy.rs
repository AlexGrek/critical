use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use persy::{Config, Persy, PersyError, PersyId};

use crate::store::StorageError;

pub type StorageResult<T> = Result<T, StorageError>;

pub trait KvStorage: Send + Sync {
    fn initialize(&mut self, store: &str) -> StorageResult<()>;
    fn get(&self, store: &str, key: &str) -> StorageResult<Vec<String>>;
    fn set(&mut self, store: &str, key: &str, value: Vec<String>) -> StorageResult<()>;
}

pub struct PersyKv {
    base_path: PathBuf,
    stores: Mutex<HashMap<String, Persy>>, // One Persy per namespace
}

impl PersyKv {
    pub fn new<P: AsRef<Path>>(base_path: P) -> StorageResult<Self> {
        fs::create_dir_all(&base_path).map_err(|e| StorageError::StorageError {
            reason: format!("Failed to create storage dir: {e}"),
        })?;
        Ok(Self {
            base_path: base_path.as_ref().to_path_buf(),
            stores: Mutex::new(HashMap::new()),
        })
    }

    fn db_path(&self, store: &str) -> PathBuf {
        self.base_path.join(format!("{store}.db"))
    }

    fn open_or_create(&self, store: &str) -> StorageResult<Persy> {
        let db_path = self.db_path(store);
        if !db_path.exists() {
            Persy::create(db_path.clone()).map_err(|e| StorageError::StorageError {
                reason: format!("Failed to create DB: {e}"),
            })?;
        }

        let persy = Persy::open(db_path.clone(), Config::new()).map_err(|e| {
            StorageError::StorageError {
                reason: format!("Failed to open DB: {e}"),
            }
        })?;

        if !persy
            .exists_segment("data")
            .map_err(|e| StorageError::StorageError {
                reason: e.persy_error().to_string(),
            })?
        {
            let mut tx = persy.begin().map_err(|e| StorageError::StorageError {
                reason: e.persy_error().to_string(),
            })?;
            tx.create_segment("data")
                .map_err(|e| map_persy(e.persy_error()))?;
            tx.commit().map_err(|e| map_persy(e.persy_error()))?;
        }

        Ok(persy)
    }

    fn get_persy(&self, store: &str) -> StorageResult<Persy> {
        let mut map = self.stores.lock().unwrap();
        if let Some(db) = map.get(store) {
            return Ok(db.clone());
        }

        let db = self.open_or_create(store)?;
        map.insert(store.to_string(), db.clone());
        Ok(db)
    }
}

impl KvStorage for PersyKv {
    fn initialize(&mut self, store: &str) -> StorageResult<()> {
        let db = self.open_or_create(store)?;
        let mut map = self.stores.lock().unwrap();
        map.insert(store.to_string(), db);
        Ok(())
    }

    fn get(&self, store: &str, key: &str) -> StorageResult<Vec<String>> {
        let db = self.get_persy(store)?;
        let mut tx = db.begin().map_err(|e| map_persy(e.persy_error()))?;

        let mut read_id = db
            .get::<String, PersyId>(store, &String::from(key))
            .map_err(|e| map_persy(e.persy_error()))?;
        if let Some(id) = read_id.next() {
            let data = tx.read(key, &id).map_err(|e| map_persy(e.persy_error()))?;
            if data.is_none() {
                return Err(StorageError::ItemNotFound {
                    key: key.to_string(),
                    kind: store.to_string(),
                });
            }
            let (parsed, _len) = bincode::serde::decode_from_slice::<
                Vec<String>,
                bincode::config::Configuration,
            >(&data.unwrap(), bincode::config::standard())
            .map_err(|e| StorageError::ReadItemFailure {
                reason: format!("{e}"),
            })?;
            return Ok(parsed);
        }

        return Err(StorageError::ItemNotFound {
            key: key.to_string(),
            kind: store.to_string(),
        });
    }

    fn set(&mut self, store: &str, key: &str, value: Vec<String>) -> StorageResult<()> {
        let db = self.get_persy(store)?;
        let mut tx = db.begin().map_err(|e| map_persy(e.persy_error()))?;

        let serialized =
            bincode::encode_to_vec(&value, bincode::config::standard()).map_err(|e| {
                StorageError::WriteItemFailure {
                    reason: format!("{e}"),
                }
            })?;

        tx.insert(key, &serialized)
            .map_err(|e| map_persy(e.persy_error()))?;

        tx.commit().map_err(|e| map_persy(e.persy_error()))?;
        Ok(())
    }
}

fn map_persy(e: PersyError) -> StorageError {
    StorageError::StorageError {
        reason: format!("Persy error: {e}"),
    }
}
