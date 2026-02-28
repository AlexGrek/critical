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

/// Server-managed audit timestamps. NOT part of desired state — excluded
/// from hash computation and not user-modifiable.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ResourceState {
    pub created_at: DateTime<Utc>,
    pub created_by: Option<PrincipalId>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<PrincipalId>,
}

/// Server-injected runtime data, NOT part of desired state or history.
/// Used for computed/dynamic fields like last_login, member_count, etc.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RuntimeState {
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
    /// Server-injected runtime data (member_count, last_login, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_state: Option<RuntimeState>,
    /// Change history (oldest first).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<HistoryEntry>>,
    /// Runtime events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<ResourceEvent>>,
}

// ---------------------------------------------------------------------------
// Media / file tracking
// ---------------------------------------------------------------------------

/// Tracks a raw uploaded image that is pending background processing.
/// Stored in `unprocessed_images` collection; hard-deleted when processing completes or fails.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnprocessedImage {
    #[serde(rename = "_key")]
    pub id: String,
    /// Original filename: `{ulid}.{ext}` (e.g. `01jz....jpg`).
    pub filename: String,
    /// User ID who uploaded the image.
    pub owner_id: String,
    /// "avatar" or "wallpaper".
    pub upload_type: String,
    pub created_at: DateTime<Utc>,
}

/// Resolved URIs (filenames, no directory prefix) for the two sizes of a processed image.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersistentFileUri {
    /// HD variant filename, e.g. `01jz..._hd.webp`.
    pub hd: String,
    /// Thumbnail variant filename, e.g. `01jz..._thumb.webp`.
    pub thumb: String,
}

/// Persistent record for a fully processed, stored image file.
/// Stored in `persistent_files` collection; survives as long as the file is live.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersistentFile {
    #[serde(rename = "_key")]
    pub id: String,
    /// Object-store subdirectory that owns this file (`user_avatars` or `user_wallpapers`).
    pub category: String,
    /// Always `"principal"` for now; reserved for future relation types.
    pub relation_type: String,
    /// Owner principal ID (e.g. `u_alice`).
    pub owner: String,
    /// Always `"webp"`.
    pub format: String,
    /// Available size variants: `["hd", "thumb"]`.
    pub sizes: Vec<String>,
    /// Combined byte size of all stored variants.
    pub total_size_bytes: u64,
    /// Full object-store paths for each variant (same order as `sizes`).
    pub filenames: Vec<String>,
    /// Convenience URIs (filenames only, without directory) for each size.
    pub uri: PersistentFileUri,
    pub created_at: DateTime<Utc>,
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
