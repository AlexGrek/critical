use std::collections::HashMap;

use chrono::{DateTime, Utc};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

pub type PrincipalId = String;

/// Label map for `-l` / `--field-selector` filtering (like kubectl).
pub type Labels = HashMap<String, String>;

bitflags! {
    // derive common traits for easier usage
    #[derive(Default, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Permissions: u8 {
        const NONE = 0;
        const FETCH    = 1 << 0; // 0000 0001
        const LIST   = 1 << 1; // 0000 0010
        const NOTIFY = 1 << 2; // 0000 0100
        const CREATE = 1 << 3;
        const MODIFY = 1 << 4; // it also auto allows deletion
        const CUSTOM1 = 1 << 5;
        const CUSTOM2 = 1 << 6;
        const READ = Self::FETCH.bits() | Self::LIST.bits() | Self::NOTIFY.bits();
        const WRITE = Self::CREATE.bits() | Self::MODIFY.bits() | Self::READ.bits();

        // You can define composite flags (shortcuts)
        const ROOT     = Self::READ.bits() | Self::WRITE.bits() | Self::CUSTOM1.bits() | Self::CUSTOM2.bits();
        const DEFAULT = Self::NONE.bits();
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AccessControlStore {
    pub list: Vec<AccessControlList>,
    pub last_mod_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AccessControlList {
    pub permissions: Permissions,
    pub principals: Vec<String>,
}

impl AccessControlStore {
    /// Check if any of the given principals has the required permission in this ACL.
    pub fn check_permission(&self, principals: &[String], required: Permissions) -> bool {
        self.list.iter().any(|acl| {
            acl.permissions.contains(required)
                && acl.principals.iter().any(|p| principals.contains(p))
        })
    }
}

/// Common metadata embedded in every resource.
/// Replaces ad-hoc `metadata: HashMap` and individual `created_at` / `created_by` fields.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ResourceMeta {
    /// Queryable key-value pairs for `-l` label selectors.
    pub labels: Labels,
    /// Non-queryable freeform annotations.
    pub annotations: Labels,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<PrincipalId>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<PrincipalId>,
}

/// Soft-delete / archive state used across all resource kinds.
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleState {
    #[default]
    Active,
    Archived,
    Deleted,
}

/// Typed relation kinds matching the whitepaper's graph edge examples.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RelationKind {
    BelongsTo,   // task -> sprint
    Implements,  // task -> feature
    CausedBy,    // bug -> release
    Deploys,     // deployment -> artifact
    BuiltFrom,   // artifact -> pipeline_run
    TriggeredBy, // pipeline -> repo
    References,  // page -> task
    Custom(String),
}

/// Generic edge document stored in the `relations` edge collection.
/// Does NOT replace `GroupMembership` — membership has specialised AQL graph traversal queries.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Relation {
    #[serde(rename = "_key")]
    pub id: String,
    #[serde(rename = "_from")]
    pub from: String, // e.g. "tasks/t_abc"
    #[serde(rename = "_to")]
    pub to: String, // e.g. "sprints/sp_xyz"
    pub kind: RelationKind,
    pub meta: ResourceMeta,
}

pub mod super_permissions {
    pub const ADM_USER_MANAGER: &str = "adm_user_manager";
    pub const ADM_PROJECT_MANAGER: &str = "adm_project_manager";
    pub const USR_CREATE_PROJECTS: &str = "usr_create_projects";
    pub const ADM_CONFIG_EDITOR: &str = "adm_config_editor";
    pub const USR_CREATE_GROUPS: &str = "usr_create_groups";
    pub const USR_CREATE_PIPELINES: &str = "usr_create_pipelines";
    pub const ADM_POLICY_EDITOR: &str = "adm_policy_editor";
    pub const ADM_RELEASE_MANAGER: &str = "adm_release_manager";
}
