use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::db::ArangoDb;
use crate::error::AppError;
use crate::middleware::auth::Auth;
use crit_shared::models::{AccessControlList, AccessControlStore, Permissions};

// ---------------------------------------------------------------------------
// KindController trait
// ---------------------------------------------------------------------------

/// Trait that each kind-specific controller implements to handle authorization
/// and document transformation for the generic gitops API.
#[async_trait]
pub trait KindController: Send + Sync {
    /// Check if a user can read a document of this kind.
    async fn can_read(&self, user_id: &str, doc: Option<&Value>) -> Result<bool, AppError>;

    /// Check if a user can write (create/update/delete) a document of this kind.
    async fn can_write(&self, user_id: &str, doc: Option<&Value>) -> Result<bool, AppError>;

    /// Convert an external gitops request body to an internal ArangoDB document.
    fn to_internal(&self, body: Value, auth: &Auth) -> Result<Value, AppError>;

    /// Convert an internal ArangoDB document to the external gitops representation.
    fn to_external(&self, doc: Value) -> Value;

    /// Check if a user can create a new document of this kind.
    /// Receives the request body so controllers can inspect the target resource
    /// (e.g. MembershipController checks the target group's ACL).
    /// Default delegates to can_write(user_id, None).
    async fn can_create(&self, user_id: &str, _body: &Value) -> Result<bool, AppError> {
        self.can_write(user_id, None).await
    }

    /// Prepare a document body before creation (e.g. inject creator ACL).
    /// Default is a no-op.
    fn prepare_create(&self, _body: &mut Value, _user_id: &str) {}

    /// Called after a document is successfully created. Used for post-creation
    /// setup (e.g. inserting creator as group member).
    /// Default is a no-op.
    async fn after_create(&self, _key: &str, _user_id: &str, _db: &ArangoDb) -> Result<(), AppError> {
        Ok(())
    }

    /// Called after a document is deleted. Used for cascade cleanup.
    /// Default is a no-op.
    async fn after_delete(&self, _key: &str, _db: &ArangoDb) -> Result<(), AppError> {
        Ok(())
    }

    /// Called after a document is updated/upserted. Used for post-update checks
    /// (e.g. empty-group deletion).
    /// Default is a no-op.
    async fn after_update(&self, _key: &str, _db: &ArangoDb) -> Result<(), AppError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Shared helpers (reusable by all controllers)
// ---------------------------------------------------------------------------

/// Rename `id` → `_key` in a JSON object.
pub fn rename_id_to_key(body: &mut Value) {
    if let Some(obj) = body.as_object_mut() {
        if let Some(id) = obj.remove("id") {
            obj.insert("_key".to_string(), id);
        }
    }
}

/// Rename `_key` → `id` and strip ArangoDB internal fields (`_id`, `_rev`).
pub fn rename_key_to_id(doc: &mut Value) {
    if let Some(obj) = doc.as_object_mut() {
        if let Some(key) = obj.remove("_key") {
            obj.insert("id".to_string(), key);
        }
        obj.remove("_id");
        obj.remove("_rev");
    }
}

/// Standard `to_internal`: renames `id` → `_key`.
pub fn standard_to_internal(mut body: Value) -> Value {
    rename_id_to_key(&mut body);
    body
}

/// Standard `to_external`: renames `_key` → `id`, strips `_id`/`_rev`.
pub fn standard_to_external(mut doc: Value) -> Value {
    rename_key_to_id(&mut doc);
    doc
}

/// Parse ACL from a raw JSON document.
/// Handles both numeric permissions (from gitops API) and string flags (from Rust serde).
pub fn parse_acl(doc: &Value) -> Result<AccessControlStore, ()> {
    let acl_val = doc.get("acl").ok_or(())?;
    let acl_obj = acl_val.as_object().ok_or(())?;

    let list = acl_obj
        .get("list")
        .and_then(|v| v.as_array())
        .ok_or(())?;

    let mut entries = Vec::new();
    for entry in list {
        let permissions = match entry.get("permissions") {
            Some(Value::Number(n)) => {
                let bits = n.as_u64().ok_or(())? as u8;
                Permissions::from_bits(bits).ok_or(())?
            }
            Some(Value::String(_)) => {
                // Bitflags 2.x serde format: "FETCH | LIST | NOTIFY"
                serde_json::from_value::<Permissions>(
                    entry.get("permissions").cloned().unwrap(),
                )
                .map_err(|_| ())?
            }
            _ => return Err(()),
        };

        let principals = entry
            .get("principals")
            .and_then(|v| v.as_array())
            .ok_or(())?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        entries.push(AccessControlList {
            permissions,
            principals,
        });
    }

    let last_mod_date = acl_obj
        .get("last_mod_date")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_default();

    Ok(AccessControlStore {
        list: entries,
        last_mod_date,
    })
}

// ---------------------------------------------------------------------------
// DefaultKindController — permissive fallback for unknown kinds
// ---------------------------------------------------------------------------

pub struct DefaultKindController;

#[async_trait]
impl KindController for DefaultKindController {
    async fn can_read(&self, _user_id: &str, _doc: Option<&Value>) -> Result<bool, AppError> {
        Ok(true)
    }

    async fn can_write(&self, _user_id: &str, _doc: Option<&Value>) -> Result<bool, AppError> {
        Ok(true)
    }

    fn to_internal(&self, body: Value, _auth: &Auth) -> Result<Value, AppError> {
        Ok(standard_to_internal(body))
    }

    fn to_external(&self, doc: Value) -> Value {
        standard_to_external(doc)
    }
}

// ---------------------------------------------------------------------------
// GitopsController — dispatch to kind-specific controllers
// ---------------------------------------------------------------------------

pub struct GitopsController {
    pub db: Arc<ArangoDb>,
}

impl GitopsController {
    pub fn new(db: Arc<ArangoDb>) -> Self {
        Self { db }
    }
}
