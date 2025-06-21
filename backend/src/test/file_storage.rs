
#[cfg(test)]
mod tests {
    use gitops_lib::{store::{FilesystemDatabaseProvider, FilesystemNamespacedDatabaseProvider, GenericDatabaseProvider, GenericNamespacedDatabaseProvider}, GitopsEnum, GitopsResourcePart, GitopsResourceRoot};
    use serde::{Deserialize, Serialize};
    use std::sync::{atomic::{AtomicUsize, Ordering}, Arc};
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
        let deployment = Deployment { name: "to-delete".to_string(), creation_timestamp: "tbd".to_string(), status: None, additional_info: None };

        db.insert(&deployment).await.unwrap();
        assert!(db.get_by_key("to-delete").await.is_ok());

        db.delete("to-delete").await.unwrap();
        assert!(db.get_by_key("to-delete").await.is_err());
    }

    #[tokio::test]
    async fn test_fs_provider_list() {
        let dir = tempdir().unwrap();
        let db = FilesystemDatabaseProvider::<Deployment>::new(dir.path(), None);
        
        db.insert(&Deployment { name: "app1".into(), ..Default::default() }).await.unwrap();
        db.insert(&Deployment { name: "app2".into(), ..Default::default() }).await.unwrap();
        
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
        let deployment = Deployment { name: "tx-app".into(), ..Default::default() };
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

        let final_item = FilesystemDatabaseProvider::<Deployment>::new(dir.path(), None).get_by_key("tx-app").await.unwrap();
        assert_eq!(final_item.additional_info, Some("success".to_string()));
    }
    
    // --- Tests for FilesystemNamespacedDatabaseProvider ---

    #[tokio::test]
    async fn test_fs_namespaced_provider_crud() {
        let dir = tempdir().unwrap();
        let db = FilesystemNamespacedDatabaseProvider::<Deployment>::new(dir.path(), None);
        let deployment = Deployment { name: "ns-app".into(), ..Default::default() };

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
        let deployment = Deployment { name: "auto-ns".into(), ..Default::default() };
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
    
}
