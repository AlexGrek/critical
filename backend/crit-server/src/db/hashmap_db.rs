use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::error::Error;
use std::fmt;

use crate::db::core::DatabaseProvider;

// Custom error type for our implementation
#[derive(Debug)]
pub struct HashMapDbError {
    message: String,
}

impl fmt::Display for HashMapDbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HashMapDb error: {}", self.message)
    }
}

impl Error for HashMapDbError {}

impl HashMapDbError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

// Transaction state - keeps track of operations within a transaction
#[derive(Debug)]
pub struct HashMapTransaction {
    // Stores operations as (key, value) pairs to be committed
    operations: HashMap<String, Option<String>>, // None means delete
    committed: bool,
    rolled_back: bool,
}

impl HashMapTransaction {
    fn new() -> Self {
        println!("🔄 Creating new transaction");
        Self {
            operations: HashMap::new(),
            committed: false,
            rolled_back: false,
        }
    }

    fn is_active(&self) -> bool {
        !self.committed && !self.rolled_back
    }
}

// Main HashMap-based database provider
pub struct HashMapDatabaseProvider {
    // Using Arc<Mutex<>> to allow sharing across async contexts
    storage: Arc<Mutex<HashMap<String, String>>>,
}

impl HashMapDatabaseProvider {
    pub fn new() -> Self {
        println!("🗄️  Initializing HashMapDatabaseProvider");
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Helper function to create storage keys
    fn make_key(resource_type: &str, namespace: Option<&str>, key: &str) -> String {
        match namespace {
            Some(ns) => format!("{}:{}:{}", resource_type, ns, key),
            None => format!("{}:{}", resource_type, key),
        }
    }

    // Helper function to serialize to YAML
    fn serialize_to_yaml<T: Serialize>(value: &T) -> Result<String, HashMapDbError> {
        serde_yaml::to_string(value)
            .map_err(|e| HashMapDbError::new(format!("YAML serialization failed: {}", e)))
    }

    // Helper function to deserialize from YAML
    fn deserialize_from_yaml<T: DeserializeOwned>(yaml_str: &str) -> Result<T, HashMapDbError> {
        serde_yaml::from_str(yaml_str)
            .map_err(|e| HashMapDbError::new(format!("YAML deserialization failed: {}", e)))
    }
}

#[async_trait]
impl DatabaseProvider for HashMapDatabaseProvider {
    type Error = HashMapDbError;
    type Transaction = HashMapTransaction;

    // Transaction Management
    async fn start_transaction(&self) -> Result<Self::Transaction, Self::Error> {
        println!("🚀 Starting new transaction");
        Ok(HashMapTransaction::new())
    }

    async fn commit_transaction(&self, mut tx: Self::Transaction) -> Result<(), Self::Error> {
        if !tx.is_active() {
            return Err(HashMapDbError::new(
                "Transaction is not active (already committed or rolled back)".to_string()
            ));
        }

        println!("✅ Committing transaction with {} operations", tx.operations.len());
        
        let mut storage = self.storage.lock().unwrap();
        
        for (key, value_opt) in tx.operations.iter() {
            match value_opt {
                Some(value) => {
                    println!("📝 COMMIT: Setting key '{}' = '{}'", key, value);
                    storage.insert(key.clone(), value.clone());
                },
                None => {
                    println!("🗑️  COMMIT: Deleting key '{}'", key);
                    storage.remove(key);
                }
            }
        }
        
        tx.committed = true;
        println!("✅ Transaction committed successfully");
        Ok(())
    }

    async fn rollback_transaction(&self, mut tx: Self::Transaction) -> Result<(), Self::Error> {
        if !tx.is_active() {
            return Err(HashMapDbError::new(
                "Transaction is not active (already committed or rolled back)".to_string()
            ));
        }

        println!("❌ Rolling back transaction with {} operations", tx.operations.len());
        tx.rolled_back = true;
        println!("❌ Transaction rolled back successfully");
        Ok(())
    }

    // Key-Value Operations
    async fn get_resource<T: DeserializeOwned + Send + Sync>(
        &self,
        resource_type: &str,
        key: &str,
        tx: Option<&mut Self::Transaction>,
    ) -> Result<Option<T>, Self::Error> {
        let storage_key = Self::make_key(resource_type, None, key);
        println!("🔍 GET: resource_type='{}', key='{}' -> storage_key='{}'", 
                resource_type, key, storage_key);

        // Check transaction first if provided
        if let Some(transaction) = tx {
            if let Some(value_opt) = transaction.operations.get(&storage_key) {
                return match value_opt {
                    Some(yaml_str) => {
                        println!("📖 Found in transaction: '{}'", yaml_str);
                        let result = Self::deserialize_from_yaml(yaml_str)?;
                        Ok(Some(result))
                    },
                    None => {
                        println!("🚫 Marked for deletion in transaction");
                        Ok(None)
                    }
                };
            }
        }

        // Check main storage
        let storage = self.storage.lock().unwrap();
        match storage.get(&storage_key) {
            Some(yaml_str) => {
                println!("📖 Found in storage: '{}'", yaml_str);
                let result = Self::deserialize_from_yaml(yaml_str)?;
                Ok(Some(result))
            },
            None => {
                println!("❌ Not found in storage");
                Ok(None)
            }
        }
    }

    async fn get_resource_ns<T: DeserializeOwned + Send + Sync>(
        &self,
        resource_type: &str,
        namespace: &str,
        key: &str,
        tx: Option<&mut Self::Transaction>,
    ) -> Result<Option<T>, Self::Error> {
        let storage_key = Self::make_key(resource_type, Some(namespace), key);
        println!("🔍 GET_NS: resource_type='{}', namespace='{}', key='{}' -> storage_key='{}'", 
                resource_type, namespace, key, storage_key);

        // Check transaction first if provided
        if let Some(transaction) = tx {
            if let Some(value_opt) = transaction.operations.get(&storage_key) {
                return match value_opt {
                    Some(yaml_str) => {
                        println!("📖 Found in transaction: '{}'", yaml_str);
                        let result = Self::deserialize_from_yaml(yaml_str)?;
                        Ok(Some(result))
                    },
                    None => {
                        println!("🚫 Marked for deletion in transaction");
                        Ok(None)
                    }
                };
            }
        }

        // Check main storage
        let storage = self.storage.lock().unwrap();
        match storage.get(&storage_key) {
            Some(yaml_str) => {
                println!("📖 Found in storage: '{}'", yaml_str);
                let result = Self::deserialize_from_yaml(yaml_str)?;
                Ok(Some(result))
            },
            None => {
                println!("❌ Not found in storage");
                Ok(None)
            }
        }
    }

    async fn set_resource<T: Serialize + Send + Sync>(
        &self,
        resource_type: &str,
        key: &str,
        value: &T,
        tx: &mut Self::Transaction,
    ) -> Result<(), Self::Error> {
        if !tx.is_active() {
            return Err(HashMapDbError::new("Transaction is not active".to_string()));
        }

        let storage_key = Self::make_key(resource_type, None, key);
        let yaml_str = Self::serialize_to_yaml(value)?;
        
        println!("📝 SET: resource_type='{}', key='{}' -> storage_key='{}', value='{}'", 
                resource_type, key, storage_key, yaml_str);

        tx.operations.insert(storage_key, Some(yaml_str));
        Ok(())
    }

    async fn set_resource_ns<T: Serialize + Send + Sync>(
        &self,
        resource_type: &str,
        namespace: &str,
        key: &str,
        value: &T,
        tx: &mut Self::Transaction,
    ) -> Result<(), Self::Error> {
        if !tx.is_active() {
            return Err(HashMapDbError::new("Transaction is not active".to_string()));
        }

        let storage_key = Self::make_key(resource_type, Some(namespace), key);
        let yaml_str = Self::serialize_to_yaml(value)?;
        
        println!("📝 SET_NS: resource_type='{}', namespace='{}', key='{}' -> storage_key='{}', value='{}'", 
                resource_type, namespace, key, storage_key, yaml_str);

        tx.operations.insert(storage_key, Some(yaml_str));
        Ok(())
    }

    async fn delete_resource(
        &self,
        resource_type: &str,
        key: &str,
        tx: &mut Self::Transaction,
    ) -> Result<Option<String>, Self::Error> {
        if !tx.is_active() {
            return Err(HashMapDbError::new("Transaction is not active".to_string()));
        }

        let storage_key = Self::make_key(resource_type, None, key);
        println!("🗑️  DELETE: resource_type='{}', key='{}' -> storage_key='{}'", 
                resource_type, key, storage_key);

        // Check if it exists first
        let storage = self.storage.lock().unwrap();
        let existing = storage.get(&storage_key).cloned();
        drop(storage);

        tx.operations.insert(storage_key, None);
        Ok(existing)
    }

    async fn delete_resource_ns(
        &self,
        resource_type: &str,
        namespace: &str,
        key: &str,
        tx: &mut Self::Transaction,
    ) -> Result<Option<String>, Self::Error> {
        if !tx.is_active() {
            return Err(HashMapDbError::new("Transaction is not active".to_string()));
        }

        let storage_key = Self::make_key(resource_type, Some(namespace), key);
        println!("🗑️  DELETE_NS: resource_type='{}', namespace='{}', key='{}' -> storage_key='{}'", 
                resource_type, namespace, key, storage_key);

        // Check if it exists first
        let storage = self.storage.lock().unwrap();
        let existing = storage.get(&storage_key).cloned();
        drop(storage);

        tx.operations.insert(storage_key, None);
        Ok(existing)
    }

    async fn list_keys(&self, resource_type: &str) -> Result<Vec<String>, Self::Error> {
        println!("📋 LIST_KEYS: resource_type='{}'", resource_type);
        
        let storage = self.storage.lock().unwrap();
        let prefix = format!("{}:", resource_type);
        
        let keys: Vec<String> = storage.keys()
            .filter(|key| key.starts_with(&prefix))
            .filter_map(|key| {
                // Extract the actual key part (after resource_type:)
                let parts: Vec<&str> = key.splitn(3, ':').collect();
                if parts.len() == 2 {
                    // Format: resource_type:key
                    Some(parts[1].to_string())
                } else {
                    // Skip namespaced keys (format: resource_type:namespace:key)
                    None
                }
            })
            .collect();

        println!("📋 Found {} keys: {:?}", keys.len(), keys);
        Ok(keys)
    }

    async fn list_keys_ns(&self, resource_type: &str, namespace: &str) -> Result<Vec<String>, Self::Error> {
        println!("📋 LIST_KEYS_NS: resource_type='{}', namespace='{}'", resource_type, namespace);
        
        let storage = self.storage.lock().unwrap();
        let prefix = format!("{}:{}:", resource_type, namespace);
        
        let keys: Vec<String> = storage.keys()
            .filter(|key| key.starts_with(&prefix))
            .filter_map(|key| {
                // Extract the actual key part (after resource_type:namespace:)
                let parts: Vec<&str> = key.splitn(3, ':').collect();
                if parts.len() == 3 {
                    Some(parts[2].to_string())
                } else {
                    None
                }
            })
            .collect();

        println!("📋 Found {} keys in namespace: {:?}", keys.len(), keys);
        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    // Test data structures
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestUser {
        id: u32,
        name: String,
        email: String,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestProduct {
        id: u32,
        name: String,
        price: f64,
        in_stock: bool,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct ComplexData {
        metadata: std::collections::HashMap<String, String>,
        tags: Vec<String>,
        nested: NestedData,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct NestedData {
        counter: i32,
        active: bool,
    }

    // Helper function to create test data
    fn create_test_user(id: u32, name: &str, email: &str) -> TestUser {
        TestUser {
            id,
            name: name.to_string(),
            email: email.to_string(),
        }
    }

    fn create_test_product(id: u32, name: &str, price: f64, in_stock: bool) -> TestProduct {
        TestProduct {
            id,
            name: name.to_string(),
            price,
            in_stock,
        }
    }

    fn create_complex_data() -> ComplexData {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("version".to_string(), "1.0".to_string());
        metadata.insert("author".to_string(), "test".to_string());

        ComplexData {
            metadata,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            nested: NestedData {
                counter: 42,
                active: true,
            },
        }
    }

    // Basic functionality tests
    #[tokio::test]
    async fn test_basic_set_and_get() {
        let db = HashMapDatabaseProvider::new();
        let mut tx = db.start_transaction().await.unwrap();

        let user = create_test_user(1, "Alice", "alice@example.com");
        
        // Set resource
        db.set_resource("users", "alice", &user, &mut tx).await.unwrap();
        
        // Get resource (should find it in transaction)
        let retrieved: Option<TestUser> = db.get_resource("users", "alice", Some(&mut tx)).await.unwrap();
        assert_eq!(retrieved, Some(user.clone()));

        // Commit transaction
        db.commit_transaction(tx).await.unwrap();

        // Get resource after commit (should find it in storage)
        let retrieved_after_commit: Option<TestUser> = db.get_resource("users", "alice", None).await.unwrap();
        assert_eq!(retrieved_after_commit, Some(user));
    }

    #[tokio::test]
    async fn test_namespaced_operations() {
        let db = HashMapDatabaseProvider::new();
        let mut tx = db.start_transaction().await.unwrap();

        let user1 = create_test_user(1, "Alice", "alice@example.com");
        let user2 = create_test_user(2, "Bob", "bob@example.com");

        // Set resources in different namespaces
        db.set_resource_ns("users", "tenant1", "alice", &user1, &mut tx).await.unwrap();
        db.set_resource_ns("users", "tenant2", "alice", &user2, &mut tx).await.unwrap();

        // Get resources from different namespaces
        let retrieved1: Option<TestUser> = db.get_resource_ns("users", "tenant1", "alice", Some(&mut tx)).await.unwrap();
        let retrieved2: Option<TestUser> = db.get_resource_ns("users", "tenant2", "alice", Some(&mut tx)).await.unwrap();

        assert_eq!(retrieved1, Some(user1));
        assert_eq!(retrieved2, Some(user2));

        db.commit_transaction(tx).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_nonexistent_resource() {
        let db = HashMapDatabaseProvider::new();
        
        let result: Option<TestUser> = db.get_resource("users", "nonexistent", None).await.unwrap();
        assert_eq!(result, None);

        let result_ns: Option<TestUser> = db.get_resource_ns("users", "tenant1", "nonexistent", None).await.unwrap();
        assert_eq!(result_ns, None);
    }

    #[tokio::test]
    async fn test_delete_operations() {
        let db = HashMapDatabaseProvider::new();
        let mut tx = db.start_transaction().await.unwrap();

        let user = create_test_user(1, "Alice", "alice@example.com");
        
        // Set and commit
        db.set_resource("users", "alice", &user, &mut tx).await.unwrap();
        db.commit_transaction(tx).await.unwrap();

        // Verify it exists
        let retrieved: Option<TestUser> = db.get_resource("users", "alice", None).await.unwrap();
        assert_eq!(retrieved, Some(user));

        // Delete in new transaction
        let mut tx2 = db.start_transaction().await.unwrap();
        let deleted = db.delete_resource("users", "alice", &mut tx2).await.unwrap();
        assert!(deleted.is_some());
        
        // Should not be found in transaction
        let retrieved_in_tx: Option<TestUser> = db.get_resource("users", "alice", Some(&mut tx2)).await.unwrap();
        assert_eq!(retrieved_in_tx, None);

        db.commit_transaction(tx2).await.unwrap();

        // Should not be found after commit
        let retrieved_after_delete: Option<TestUser> = db.get_resource("users", "alice", None).await.unwrap();
        assert_eq!(retrieved_after_delete, None);
    }

    #[tokio::test]
    async fn test_delete_namespaced_operations() {
        let db = HashMapDatabaseProvider::new();
        let mut tx = db.start_transaction().await.unwrap();

        let user = create_test_user(1, "Alice", "alice@example.com");
        
        // Set and commit
        db.set_resource_ns("users", "tenant1", "alice", &user, &mut tx).await.unwrap();
        db.commit_transaction(tx).await.unwrap();

        // Delete in new transaction
        let mut tx2 = db.start_transaction().await.unwrap();
        let deleted = db.delete_resource_ns("users", "tenant1", "alice", &mut tx2).await.unwrap();
        assert!(deleted.is_some());
        
        db.commit_transaction(tx2).await.unwrap();

        // Should not be found after commit
        let retrieved: Option<TestUser> = db.get_resource_ns("users", "tenant1", "alice", None).await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_delete_nonexistent_resource() {
        let db = HashMapDatabaseProvider::new();
        let mut tx = db.start_transaction().await.unwrap();

        let deleted = db.delete_resource("users", "nonexistent", &mut tx).await.unwrap();
        assert_eq!(deleted, None);

        let deleted_ns = db.delete_resource_ns("users", "tenant1", "nonexistent", &mut tx).await.unwrap();
        assert_eq!(deleted_ns, None);

        db.commit_transaction(tx).await.unwrap();
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        let db = HashMapDatabaseProvider::new();
        let mut tx = db.start_transaction().await.unwrap();

        let user = create_test_user(1, "Alice", "alice@example.com");
        
        // Set resource in transaction
        db.set_resource("users", "alice", &user, &mut tx).await.unwrap();
        
        // Verify it's found in transaction
        let retrieved_in_tx: Option<TestUser> = db.get_resource("users", "alice", Some(&mut tx)).await.unwrap();
        assert_eq!(retrieved_in_tx, Some(user));

        // Rollback transaction
        db.rollback_transaction(tx).await.unwrap();

        // Should not be found after rollback
        let retrieved_after_rollback: Option<TestUser> = db.get_resource("users", "alice", None).await.unwrap();
        assert_eq!(retrieved_after_rollback, None);
    }

    #[tokio::test]
    async fn test_transaction_isolation() {
        let db = HashMapDatabaseProvider::new();
        
        // Start two transactions
        let mut tx1 = db.start_transaction().await.unwrap();
        let mut tx2 = db.start_transaction().await.unwrap();

        let user1 = create_test_user(1, "Alice", "alice@example.com");
        let user2 = create_test_user(2, "Bob", "bob@example.com");

        // Set different resources in each transaction
        db.set_resource("users", "alice", &user1, &mut tx1).await.unwrap();
        db.set_resource("users", "bob", &user2, &mut tx2).await.unwrap();

        // Each transaction should only see its own changes
        let alice_in_tx1: Option<TestUser> = db.get_resource("users", "alice", Some(&mut tx1)).await.unwrap();
        let bob_in_tx1: Option<TestUser> = db.get_resource("users", "bob", Some(&mut tx1)).await.unwrap();
        let alice_in_tx2: Option<TestUser> = db.get_resource("users", "alice", Some(&mut tx2)).await.unwrap();
        let bob_in_tx2: Option<TestUser> = db.get_resource("users", "bob", Some(&mut tx2)).await.unwrap();

        assert_eq!(alice_in_tx1, Some(user1));
        assert_eq!(bob_in_tx1, None); // tx1 doesn't see tx2's changes
        assert_eq!(alice_in_tx2, None); // tx2 doesn't see tx1's changes
        assert_eq!(bob_in_tx2, Some(user2));

        // Commit both transactions
        db.commit_transaction(tx1).await.unwrap();
        db.commit_transaction(tx2).await.unwrap();

        // Now both should be visible
        let alice_final: Option<TestUser> = db.get_resource("users", "alice", None).await.unwrap();
        let bob_final: Option<TestUser> = db.get_resource("users", "bob", None).await.unwrap();

        assert!(alice_final.is_some());
        assert!(bob_final.is_some());
    }

    #[tokio::test]
    async fn test_list_keys() {
        let db = HashMapDatabaseProvider::new();
        let mut tx = db.start_transaction().await.unwrap();

        let user1 = create_test_user(1, "Alice", "alice@example.com");
        let user2 = create_test_user(2, "Bob", "bob@example.com");
        let product = create_test_product(1, "Widget", 19.99, true);

        // Set resources
        db.set_resource("users", "alice", &user1, &mut tx).await.unwrap();
        db.set_resource("users", "bob", &user2, &mut tx).await.unwrap();
        db.set_resource("products", "widget", &product, &mut tx).await.unwrap();

        db.commit_transaction(tx).await.unwrap();

        // List keys for users
        let user_keys = db.list_keys("users").await.unwrap();
        assert_eq!(user_keys.len(), 2);
        assert!(user_keys.contains(&"alice".to_string()));
        assert!(user_keys.contains(&"bob".to_string()));

        // List keys for products
        let product_keys = db.list_keys("products").await.unwrap();
        assert_eq!(product_keys.len(), 1);
        assert!(product_keys.contains(&"widget".to_string()));

        // List keys for non-existent resource type
        let empty_keys = db.list_keys("nonexistent").await.unwrap();
        assert_eq!(empty_keys.len(), 0);
    }

    #[tokio::test]
    async fn test_list_keys_namespaced() {
        let db = HashMapDatabaseProvider::new();
        let mut tx = db.start_transaction().await.unwrap();

        let user1 = create_test_user(1, "Alice", "alice@example.com");
        let user2 = create_test_user(2, "Bob", "bob@example.com");
        let user3 = create_test_user(3, "Charlie", "charlie@example.com");

        // Set resources in different namespaces
        db.set_resource_ns("users", "tenant1", "alice", &user1, &mut tx).await.unwrap();
        db.set_resource_ns("users", "tenant1", "bob", &user2, &mut tx).await.unwrap();
        db.set_resource_ns("users", "tenant2", "charlie", &user3, &mut tx).await.unwrap();

        db.commit_transaction(tx).await.unwrap();

        // List keys for tenant1
        let tenant1_keys = db.list_keys_ns("users", "tenant1").await.unwrap();
        assert_eq!(tenant1_keys.len(), 2);
        assert!(tenant1_keys.contains(&"alice".to_string()));
        assert!(tenant1_keys.contains(&"bob".to_string()));

        // List keys for tenant2
        let tenant2_keys = db.list_keys_ns("users", "tenant2").await.unwrap();
        assert_eq!(tenant2_keys.len(), 1);
        assert!(tenant2_keys.contains(&"charlie".to_string()));

        // List keys for non-existent namespace
        let empty_keys = db.list_keys_ns("users", "nonexistent").await.unwrap();
        assert_eq!(empty_keys.len(), 0);
    }

    #[tokio::test]
    async fn test_complex_data_serialization() {
        let db = HashMapDatabaseProvider::new();
        let mut tx = db.start_transaction().await.unwrap();

        let complex_data = create_complex_data();
        
        // Set complex data
        db.set_resource("complex", "test", &complex_data, &mut tx).await.unwrap();
        
        // Get complex data
        let retrieved: Option<ComplexData> = db.get_resource("complex", "test", Some(&mut tx)).await.unwrap();
        assert_eq!(retrieved, Some(complex_data));

        db.commit_transaction(tx).await.unwrap();
    }

    #[tokio::test]
    async fn test_transaction_error_handling() {
        let db = HashMapDatabaseProvider::new();
        let mut tx = db.start_transaction().await.unwrap();

        // Commit transaction
        db.commit_transaction(tx).await.unwrap();

        // Try to use the committed transaction (should fail)
        let mut tx2 = db.start_transaction().await.unwrap();
        db.commit_transaction(tx2).await.unwrap();

        // Create a new transaction for testing
        let mut tx3 = db.start_transaction().await.unwrap();
        let user = create_test_user(1, "Alice", "alice@example.com");

        // This should work
        let result = db.set_resource("users", "alice", &user, &mut tx3).await;
        assert!(result.is_ok());

        // Rollback the transaction
        db.rollback_transaction(tx3).await.unwrap();

        // Create another transaction and try to use the rolled back one
        let mut tx4 = db.start_transaction().await.unwrap();
        let result2 = db.set_resource("users", "alice", &user, &mut tx4).await;
        assert!(result2.is_ok());

        db.commit_transaction(tx4).await.unwrap();
    }

    #[tokio::test]
    async fn test_multiple_resource_types() {
        let db = HashMapDatabaseProvider::new();
        let mut tx = db.start_transaction().await.unwrap();

        let user = create_test_user(1, "Alice", "alice@example.com");
        let product = create_test_product(1, "Widget", 19.99, true);

        // Set different resource types with same key
        db.set_resource("users", "item1", &user, &mut tx).await.unwrap();
        db.set_resource("products", "item1", &product, &mut tx).await.unwrap();

        // Get both resources
        let retrieved_user: Option<TestUser> = db.get_resource("users", "item1", Some(&mut tx)).await.unwrap();
        let retrieved_product: Option<TestProduct> = db.get_resource("products", "item1", Some(&mut tx)).await.unwrap();

        assert_eq!(retrieved_user, Some(user));
        assert_eq!(retrieved_product, Some(product));

        db.commit_transaction(tx).await.unwrap();
    }

    #[tokio::test]
    async fn test_overwrite_resource() {
        let db = HashMapDatabaseProvider::new();
        let mut tx = db.start_transaction().await.unwrap();

        let user1 = create_test_user(1, "Alice", "alice@example.com");
        let user2 = create_test_user(2, "Alice Updated", "alice.updated@example.com");

        // Set initial resource
        db.set_resource("users", "alice", &user1, &mut tx).await.unwrap();
        
        // Verify initial resource
        let retrieved1: Option<TestUser> = db.get_resource("users", "alice", Some(&mut tx)).await.unwrap();
        assert_eq!(retrieved1, Some(user1));

        // Overwrite with new resource
        db.set_resource("users", "alice", &user2, &mut tx).await.unwrap();
        
        // Verify overwritten resource
        let retrieved2: Option<TestUser> = db.get_resource("users", "alice", Some(&mut tx)).await.unwrap();
        assert_eq!(retrieved2, Some(user2));

        db.commit_transaction(tx).await.unwrap();
    }

    #[tokio::test]
    async fn test_transaction_with_mixed_operations() {
        let db = HashMapDatabaseProvider::new();
        
        // First, set up some initial data
        let mut setup_tx = db.start_transaction().await.unwrap();
        let user1 = create_test_user(1, "Alice", "alice@example.com");
        let user2 = create_test_user(2, "Bob", "bob@example.com");
        
        db.set_resource("users", "alice", &user1, &mut setup_tx).await.unwrap();
        db.set_resource("users", "bob", &user2, &mut setup_tx).await.unwrap();
        db.commit_transaction(setup_tx).await.unwrap();

        // Now perform mixed operations in a single transaction
        let mut tx = db.start_transaction().await.unwrap();
        let user3 = create_test_user(3, "Charlie", "charlie@example.com");
        
        // Add new user
        db.set_resource("users", "charlie", &user3, &mut tx).await.unwrap();
        
        // Delete existing user
        db.delete_resource("users", "bob", &mut tx).await.unwrap();
        
        // Update existing user
        let updated_alice = create_test_user(1, "Alice Updated", "alice.new@example.com");
        db.set_resource("users", "alice", &updated_alice, &mut tx).await.unwrap();

        // Verify state within transaction
        let alice_in_tx: Option<TestUser> = db.get_resource("users", "alice", Some(&mut tx)).await.unwrap();
        let bob_in_tx: Option<TestUser> = db.get_resource("users", "bob", Some(&mut tx)).await.unwrap();
        let charlie_in_tx: Option<TestUser> = db.get_resource("users", "charlie", Some(&mut tx)).await.unwrap();

        assert_eq!(alice_in_tx, Some(updated_alice.clone()));
        assert_eq!(bob_in_tx, None); // Deleted
        assert_eq!(charlie_in_tx, Some(user3.clone()));

        db.commit_transaction(tx).await.unwrap();

        // Verify final state
        let alice_final: Option<TestUser> = db.get_resource("users", "alice", None).await.unwrap();
        let bob_final: Option<TestUser> = db.get_resource("users", "bob", None).await.unwrap();
        let charlie_final: Option<TestUser> = db.get_resource("users", "charlie", None).await.unwrap();

        assert_eq!(alice_final, Some(updated_alice));
        assert_eq!(bob_final, None);
        assert_eq!(charlie_final, Some(user3));
    }
}
