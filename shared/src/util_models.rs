use std::collections::HashMap;

use bitflags::bitflags;
use chrono::{DateTime, Utc};
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
    /// Service-kind scope for project ACL entries (e.g. "tasks", "deployments", "*").
    /// `None` or `"*"` means applies to all service kinds (wildcard).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

impl AccessControlStore {
    /// Check if any of the given principals has the required permission in this ACL.
    pub fn check_permission(&self, principals: &[String], required: Permissions) -> bool {
        self.list.iter().any(|acl| {
            acl.permissions.contains(required)
                && acl.principals.iter().any(|p| principals.contains(p))
        })
    }

    /// Check permission filtered by scope. An ACL entry matches if:
    /// - Its scope is None (wildcard / legacy), or
    /// - Its scope is `"*"`, or
    /// - Its scope equals `resource_kind`
    pub fn check_permission_scoped(
        &self,
        principals: &[String],
        required: Permissions,
        resource_kind: &str,
    ) -> bool {
        self.list.iter().any(|acl| {
            let scope_matches = match &acl.scope {
                None => true,
                Some(s) if s == "*" => true,
                Some(s) => s == resource_kind,
            };
            scope_matches
                && acl.permissions.contains(required)
                && acl.principals.iter().any(|p| principals.contains(p))
        })
    }
}

/// Common metadata embedded in every resource.
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

// ---------------------------------------------------------------------------
// Soft deletion (replaces LifecycleState)
// ---------------------------------------------------------------------------

/// Soft-deletion marker. Present = deleted, absent = active.
/// Every GET query should filter `doc.deletion == null` by default.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeletionInfo {
    pub deleted_at: DateTime<Utc>,
    pub deleted_by: PrincipalId,
    /// Edges that were disconnected during deletion, for possible restoration.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disconnected_edges: Vec<DisconnectedEdge>,
}

/// Record of a graph edge removed during soft deletion, to support restore.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisconnectedEdge {
    pub collection: String,
    pub key: String,
    pub from: String,
    pub to: String,
}

// ---------------------------------------------------------------------------
// Resource state (server-injected, not part of desired state)
// ---------------------------------------------------------------------------

/// Server-injected runtime state, NOT part of desired state or history.
/// Used for computed/dynamic fields like last_login, member_count, etc.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ResourceState {
    #[serde(flatten)]
    pub fields: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Change history
// ---------------------------------------------------------------------------

/// Immutable snapshot of a resource's desired state at a point in time.
/// Stored in `resource_history` collection, survives resource deletion.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    #[serde(rename = "_key")]
    pub id: String, // "{collection}_{resource_key}_{revision}"
    pub resource_kind: String,
    pub resource_key: String,
    pub revision: u64,
    /// Full desired-state JSON at this point in time.
    pub snapshot: serde_json::Value,
    pub changed_by: PrincipalId,
    pub changed_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Events (runtime, not meta changes)
// ---------------------------------------------------------------------------

/// Runtime event associated with a resource.
/// Stored in `resource_events` collection, survives resource deletion.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceEvent {
    #[serde(rename = "_key")]
    pub id: String, // UUID v7
    pub resource_kind: String,
    pub resource_key: String,
    pub event_type: String, // e.g., "sign_in"
    pub timestamp: DateTime<Utc>,
    pub actor: Option<PrincipalId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Full resource envelope (for describe / full fetch)
// ---------------------------------------------------------------------------

/// Envelope for full resource representation with optional extras.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FullResource {
    /// The resource itself (desired state + metadata), flattened.
    #[serde(flatten)]
    pub resource: serde_json::Value,
    /// Server-injected runtime state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<ResourceState>,
    /// Change history (oldest first).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<HistoryEntry>>,
    /// Runtime events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<ResourceEvent>>,
}

// ---------------------------------------------------------------------------
// Super-permissions
// ---------------------------------------------------------------------------

pub mod super_permissions {
    pub const ADM_GODMODE: &str = "adm_godmode";
    pub const ADM_USER_MANAGER: &str = "adm_user_manager";
    pub const ADM_CONFIG_EDITOR: &str = "adm_config_editor";
    pub const USR_CREATE_GROUPS: &str = "usr_create_groups";
    pub const USR_CREATE_PROJECTS: &str = "usr_create_projects";
}
