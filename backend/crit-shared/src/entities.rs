use chrono::{DateTime, Utc};
use gitops_lib::{GitopsEnum, GitopsResourcePart, GitopsResourceRoot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(GitopsResourceRoot, Debug, Serialize, Deserialize, Clone, Default)]
#[gitops(key = "uid")]
pub struct UserPublicData {
    pub uid: String,
    pub email: String,
    pub annotations: HashMap<String, String>,
    pub has_admin_status: bool,
}

#[derive(GitopsResourceRoot, Debug, Serialize, Deserialize, Clone)]
#[gitops(key = "uid")]
pub struct User {
    pub uid: String,
    pub email: String,
    pub password_hash: Option<String>,
    pub oauth: Option<String>,
    pub created_at: String,
    pub annotations: HashMap<String, String>,
    pub has_admin_status: bool,
}

impl User {
    pub fn to_public_data(&self) -> UserPublicData {
        return UserPublicData { uid: self.uid.clone(), email: self.email.clone(), annotations: self.annotations.clone(), has_admin_status: self.has_admin_status }
    }
}

impl Default for User {
    fn default() -> Self {
        User {
            uid: String::new(),
            email: String::new(),
            password_hash: None,
            oauth: None,
            created_at: Utc::now().to_rfc3339(), // Provide a default or current timestamp
            annotations: HashMap::new(),
            has_admin_status: false,
        }
    }
}

#[derive(GitopsResourcePart, Debug, Deserialize, Serialize, Clone)]
pub struct VisibilityConfig {
    pub public_visible: bool,
    pub public_can_report: bool,
    pub public_can_see_tickets: Vec<String>,
}

impl Default for VisibilityConfig {
    fn default() -> Self {
        VisibilityConfig {
            public_visible: false,
            public_can_report: false,
            public_can_see_tickets: Vec::new(),
        }
    }
}

#[derive(GitopsResourcePart, Debug, Deserialize, Serialize, Clone)]
pub struct ProjectLinks {
    pub github: String,
}

impl Default for ProjectLinks {
    fn default() -> Self {
        ProjectLinks {
            github: String::new(),
        }
    }
}

#[derive(GitopsResourcePart, Debug, Deserialize, Serialize, Clone)]
pub struct ProjectTicketCategory {
    #[serde(default = "default_supported_statuses")]
    pub supported_statuses: Vec<String>,
}

fn default_supported_statuses() -> Vec<String> {
    vec![
        "Open".to_string(),
        "In Progress".to_string(),
        "Resolved".to_string(),
        "Closed".to_string(),
        "Reopened".to_string(),
        "To Do".to_string(),
        "Done".to_string(),
        "Blocked".to_string(),
    ]
}

impl Default for ProjectTicketCategory {
    fn default() -> Self {
        ProjectTicketCategory {
            supported_statuses: default_supported_statuses(),
        }
    }
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
    pub links: ProjectLinks,
    pub ticket_categories: HashMap<String, ProjectTicketCategory>,
    pub pipelines_feature_enabled: bool,
    pub releases_feature_enabled: bool,
    pub short_description: String,
    pub readme: String,
}

impl Default for Project {
    fn default() -> Self {
        Project {
            name_id: String::new(),
            public_name: String::new(),
            owner_uid: String::new(),
            admins_uid: Vec::new(),
            visibility: VisibilityConfig::default(),
            links: ProjectLinks::default(),
            ticket_categories: HashMap::new(),
            pipelines_feature_enabled: false,
            releases_feature_enabled: false,
            short_description: String::new(),
            readme: String::new(),
        }
    }
}

#[derive(GitopsResourceRoot, Debug, Serialize, Deserialize, Clone)]
#[gitops(key = "invite_uid")]
pub struct Invite {
    pub invite_uid: String,
    pub invite_key: String,
    pub used: bool,
}

impl Default for Invite {
    fn default() -> Self {
        Invite {
            invite_uid: String::new(),
            invite_key: String::new(),
            used: false,
        }
    }
}

#[derive(GitopsResourcePart, Debug, Serialize, Deserialize, Clone, Default)]

pub struct AttachmentHandle {
    pub a_type: String,
    pub is_image: bool,
    pub id: String
}

#[derive(GitopsResourceRoot, Debug, Serialize, Deserialize, Clone, Default)]
#[gitops(key = "uid")]
pub struct Ticket {
    pub uid: String,
    pub assignee: Vec<String>,
    pub reporter: String,
    pub subscribers: Vec<String>,
    pub epic: Option<String>,
    pub sprint: Option<String>,
    pub closed: bool,
    pub status: String,

    // info
    pub name: String,
    pub descr: String,
    pub attachments: Vec<AttachmentHandle>,

    // relationships
    pub duplicate_of: Option<String>,
    pub blocked_by: Option<String>,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub is_draft: bool
}

#[derive(GitopsEnum, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum UserStatus {
    Fired,
    Replaced,
    Normal,
}

impl Default for UserStatus {
    fn default() -> Self {
        UserStatus::Normal // Set a sensible default for the enum
    }
}
