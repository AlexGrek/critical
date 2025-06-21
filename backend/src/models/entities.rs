// In your main application
use chrono::{DateTime, Utc};

use gitops_lib::{GitopsEnum, GitopsResourcePart, GitopsResourceRoot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// (Include the MockDb, DatabaseProvider, and QueryableResource trait definitions here)

#[derive(GitopsResourceRoot, Debug, Serialize, Deserialize, Clone)]
#[gitops(key = "uid")]
pub struct User {
    pub uid: String,
    pub password_hash: Option<String>,
    pub oauth: Option<String>,
    pub created_at: String,
    pub annotations: HashMap<String, String>,
    pub has_admin_status: bool
}

#[derive(GitopsResourcePart, Debug, Deserialize, Serialize, Clone)]
pub struct VisibilityConfig {
    pub public_visible: bool,
    pub public_can_report: bool,
    pub public_can_see_tickets: Vec<String>
}

#[derive(GitopsResourcePart, Debug, Deserialize, Serialize, Clone)]
pub struct ProjectLinks {
    pub github: String,
}

#[derive(GitopsResourceRoot, Debug, Serialize, Deserialize, Clone)]
#[gitops(key = "name_id")]
pub struct Project {
    pub name_id: String,
    pub public_name: String,
    #[gitops(skip_on_update)]
    pub owner_uid: String,
    pub admins_uid: Vec<String>,
    pub visibility: VisibilityConfig,
    pub links: ProjectLinks
}


#[derive(GitopsEnum, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum UserStatus {
    Fired, Replaced, Normal
}
