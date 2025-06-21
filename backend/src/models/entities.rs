// In your main application
use chrono::{DateTime, Utc};

use gitops_lib::{GitopsEnum, GitopsResourcePart, GitopsResourceRoot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::db::{core::DatabaseProvider};

// (Include the MockDb, DatabaseProvider, and QueryableResource trait definitions here)


#[derive(GitopsResourcePart, Debug, Deserialize, Serialize, Clone)]
pub struct Status {
    pub ready_replicas: u32,
    pub available_replicas: u32,
    pub conditions: Vec<String>,
}

#[derive(GitopsEnum, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum UserStatus {
    Fired, Replaced, Normal
}




/// The root GitOps resource for a Deployment.
#[derive(GitopsResourceRoot, Debug, Serialize, Deserialize, Clone)]
#[gitops(key = "name", api_version = "apps.example.com/v1")] // 'name' is the key, custom apiVersion
pub struct Deployment {
    pub name: String, // This field is identified as the 'key'
    #[gitops(skip_on_update)] // This field will not be updated by merge operations
    pub creation_timestamp: String,
    pub status: Option<UserStatus>, // Optional status field
    pub additional_info: Option<String>,
}


#[derive(GitopsResourceRoot, Debug, Clone)]
#[gitops(key="email")]
pub struct User {
    pub email: String,
    
    pub metadata: HashMap<String, String>,
    
    // pub admin: Option<AdminRole>,

    pub password_hash: Option<String>,
}
