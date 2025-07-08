use crate::models::entities::{
    Deployment, User,
    UserStatus, Status // Assuming Status is used elsewhere or might be in a Deployment in future
};
use anyhow::Result;
use gitops_lib::{store::{FilesystemDatabaseProvider, GenericDatabaseProvider}, GitopsResourceRoot};
use chrono::{DateTime, Utc};
use std::{collections::HashMap, path::{Path, PathBuf}};

// You would need to make these visible for testing, e.g., by adding `pub` or moving tests to the same module
// use crate::db::core::{GenericDatabaseProvider}; // Assuming GenericDatabaseProvider is in db::core
// use crate::GitopsResourceRoot; // Already used above

#[tokio::test]
async fn test_deployment_crud_operations() -> Result<()> {
    let dir = PathBuf::from("test_data".to_string()).join("test_deployment_crud");
    tokio::fs::remove_dir_all(&dir).await;
    let db_provider = FilesystemDatabaseProvider::new(dir.as_path());

    // 1. Test Insert
    let deployment1 = Deployment {
        name: "test-deployment-1".to_string(),
        creation_timestamp: Utc::now().to_rfc3339(),
        status: Some(UserStatus::Normal),
        additional_info: None,
    };
    db_provider.insert(deployment1.clone()).await?;

    // Verify insertion by getting by key
    let fetched_deployment1: Deployment = db_provider.get_by_key("test-deployment-1").await?;
    assert_eq!(fetched_deployment1.name, "test-deployment-1");
    assert_eq!(fetched_deployment1.status, Some(UserStatus::Normal));

    // Try inserting again, should fail
    let insert_result = db_provider.insert(deployment1.clone()).await;
    assert!(insert_result.is_err());
    assert!(insert_result.unwrap_err().to_string().contains("already exists"));

    // 2. Test Upsert (update existing)
    let deployment1_updated = Deployment {
        name: "test-deployment-1".to_string(), // Key must match
        creation_timestamp: Utc::now().to_rfc3339(), // This field is #[gitops(skip_on_update)] but we provide it
        status: Some(UserStatus::Fired), // Update status
        additional_info: Some("Updated info".to_string()),
    };
    db_provider.upsert(deployment1_updated.clone()).await?;

    let fetched_updated_deployment1: Deployment = db_provider.get_by_key("test-deployment-1").await?;
    assert_eq!(fetched_updated_deployment1.name, "test-deployment-1");
    // Ensure status is updated
    assert_eq!(fetched_updated_deployment1.status, Some(UserStatus::Fired));
    // Ensure additional_info is updated
    assert_eq!(fetched_updated_deployment1.additional_info, Some("Updated info".to_string()));
    // creation_timestamp is not directly updated by GitopsResourceRoot::with_updates_from,
    // but upsert replaces the whole file, so it will reflect the new value if provided.
    // However, the macro's `skip_on_update` would apply when using `with_updates_from` for merges.
    // For `upsert`, it's a full replacement.

    // 3. Test List
    let deployment2 = Deployment {
        name: "test-deployment-2".to_string(),
        creation_timestamp: Utc::now().to_rfc3339(),
        status: None,
        additional_info: None,
    };
    db_provider.insert(deployment2).await?;

    let all_deployments = db_provider.list().await?;
    println!("All deployments: {:?}", all_deployments);
    assert_eq!(all_deployments.len(), 2);
    // Check if both deployments are present (order might not be guaranteed)
    assert!(all_deployments.iter().any(|d: &Deployment| d.name == "test-deployment-1"));
    assert!(all_deployments.iter().any(|d: &Deployment| d.name == "test-deployment-2"));

    // 4. Test List Keys
    let keys = <FilesystemDatabaseProvider as GenericDatabaseProvider<Deployment>>::list_keys(&db_provider).await?;
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"test-deployment-1".to_string()));
    assert!(keys.contains(&"test-deployment-2".to_string()));

    // 5. Test Delete
    <FilesystemDatabaseProvider as GenericDatabaseProvider<Deployment>>::delete(&db_provider, "test-deployment-1").await?;
    let remaining_deployments: Vec<Deployment> = db_provider.list().await?;
    assert_eq!(remaining_deployments.len(), 1);
    assert_eq!(remaining_deployments[0].name, "test-deployment-2");

    // Try getting deleted item, should fail
    let get_deleted_result: Result<Deployment> = db_provider.get_by_key("test-deployment-1").await;
    assert!(get_deleted_result.is_err());
    assert!(get_deleted_result.unwrap_err().to_string().contains("No such file or directory"));


    // Try deleting non-existent item, should fail gracefully or with a specific error
    let delete_non_existent_result = <FilesystemDatabaseProvider as GenericDatabaseProvider<Deployment>>::delete(&db_provider, "non-existent-deployment").await;
    assert!(delete_non_existent_result.is_err());
    assert!(delete_non_existent_result.unwrap_err().to_string().contains("not found"));


    Ok(())
}

#[tokio::test]
async fn test_user_crud_operations() -> Result<()> {
    let dir = PathBuf::from("test_data".to_string());
    tokio::fs::remove_dir_all(&dir).await;
    let db_provider = FilesystemDatabaseProvider::new(dir.as_path());

    // 1. Test Insert
    let user1 = User {
        email: "test@example.com".to_string(),
        metadata: HashMap::from([
            ("role".to_string(), "admin".to_string()),
            ("department".to_string(), "IT".to_string()),
        ]),
        password_hash: Some("somehash123".to_string()),
    };
    db_provider.insert(user1.clone()).await?;

    let fetched_user1: User = db_provider.get_by_key("test@example.com").await?;
    assert_eq!(fetched_user1.email, "test@example.com");
    assert_eq!(fetched_user1.metadata.get("role"), Some(&"admin".to_string()));

    // 2. Test Upsert (update existing)
    let user1_updated = User {
        email: "test@example.com".to_string(),
        metadata: HashMap::from([
            ("role".to_string(), "editor".to_string()), // Changed role
            ("location".to_string(), "NYC".to_string()), // Added new field
        ]),
        password_hash: None, // Removed password hash
    };
    db_provider.upsert(user1_updated).await?;

    let fetched_updated_user1: User = db_provider.get_by_key("test@example.com").await?;
    assert_eq!(fetched_updated_user1.email, "test@example.com");
    assert_eq!(fetched_updated_user1.metadata.get("role"), Some(&"editor".to_string()));
    assert_eq!(fetched_updated_user1.metadata.get("department"), None); // Old field removed if not present in new map
    assert_eq!(fetched_updated_user1.metadata.get("location"), Some(&"NYC".to_string()));
    assert_eq!(fetched_updated_user1.password_hash, None);

    // 3. Test List & Delete
    let user2 = User {
        email: "another@example.com".to_string(),
        metadata: HashMap::new(),
        password_hash: None,
    };
    db_provider.insert(user2).await?;

    let all_users: Vec<User> = db_provider.list().await?;
    assert_eq!(all_users.len(), 2);

    <FilesystemDatabaseProvider as GenericDatabaseProvider<User>>::delete(&db_provider, "test@example.com").await?;
    let remaining_users: Vec<User> = db_provider.list().await?;
    assert_eq!(remaining_users.len(), 1);
    assert_eq!(remaining_users[0].email, "another@example.com");

    Ok(())
}

// Helper to assert that a resource directory is created and exists
#[tokio::test]
async fn test_directory_creation() -> Result<()> {
    let dir = PathBuf::from("test_data".to_string());
    tokio::fs::remove_dir_all(&dir).await;
    let db_provider = FilesystemDatabaseProvider::new(&dir);

    let deployment = Deployment {
        name: "dir-test-deployment".to_string(),
        creation_timestamp: Utc::now().to_rfc3339(),
        status: None,
        additional_info: None,
    };

    // Before insert, the directory should not exist
    assert!(!dir.join(Deployment::kind()).exists());

    db_provider.insert(deployment).await?;

    // After insert, the directory should exist
    assert!(dir.join(Deployment::kind()).exists());
    assert!(dir.join(Deployment::kind()).join("dir-test-deployment.yaml").exists());

    Ok(())
}
