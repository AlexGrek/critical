use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use crate::db::ArangoDb;
use crate::error::AppError;
use crate::middleware::auth::Auth;
use crit_shared::util_models::{AccessControlList, AccessControlStore, Permissions};

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

    /// Prepare a document body before creation. The default implementation
    /// injects common fields (labels, annotations, state audit timestamps).
    /// Override to add kind-specific setup (e.g. ACL), but call
    /// `inject_create_defaults(body, user_id)` first.
    fn prepare_create(&self, body: &mut Value, user_id: &str) {
        inject_create_defaults(body, user_id);
    }

    /// Called after a document is successfully created. Used for post-creation
    /// setup (e.g. inserting creator as group member).
    /// Default is a no-op.
    async fn after_create(&self, _key: &str, _user_id: &str, _db: &ArangoDb) -> Result<(), AppError> {
        Ok(())
    }

    /// Called after a document is deleted. Used for cascade cleanup.
    /// Default is a no-op.
    async fn after_delete(&self, _key: &str, _db: &ArangoDb) -> Result<(), AppError> {
        // TODO: log any errors here explicitly, as after_delete may break data integrity and should be treated as major error if it does
        Ok(())
    }

    /// Called after a document is updated/upserted. Used for post-update checks
    /// (e.g. empty-group deletion).
    /// Default is a no-op.
    async fn after_update(&self, _key: &str, _db: &ArangoDb) -> Result<(), AppError> {
        // TODO: if it can fail, log it explicitly, as after_update may break data integrity and should be treated as major error if it does
        Ok(())
    }

    /// Convert an internal ArangoDB document to the external representation
    /// suitable for list responses (brief/summary view).
    /// Default delegates to `to_external`.
    fn to_list_external(&self, doc: Value) -> Value {
        self.to_external(doc)
    }

    /// Return the ArangoDB field names to fetch for list queries (projection).
    /// `None` means fetch all fields (no projection).
    /// Fields should use ArangoDB names (e.g. `_key`, not `id`).
    fn list_projection_fields(&self) -> Option<&'static [&'static str]> {
        None
    }

    /// Whether this resource kind is project-scoped.
    /// Scoped resources live under `/v1/projects/{project}/{kind}`.
    fn is_scoped(&self) -> bool {
        false
    }

    /// The service kind name used for ACL scope matching on the parent project.
    /// Only meaningful when `is_scoped()` returns true (e.g. "tasks", "deployments").
    fn resource_kind_name(&self) -> &str {
        ""
    }

    /// Super-permission that short-circuits ACL checks for this kind.
    /// Return `None` to indicate no super-permission bypass (fully permissive for list).
    fn super_permission(&self) -> Option<&str> {
        None
    }

    /// Bitmask for READ permission used in AQL-level filtering.
    fn read_permission_bits(&self) -> u8 {
        Permissions::READ.bits()
    }

    /// Bitmask for WRITE permission used in AQL-level filtering.
    fn write_permission_bits(&self) -> u8 {
        Permissions::MODIFY.bits()
    }

    /// Check hybrid ACL permission for a single document.
    /// Uses the resource's own ACL if non-empty, otherwise falls back to
    /// the project's ACL filtered by scope.
    fn check_hybrid_acl(
        &self,
        doc: &Value,
        principals: &[String],
        required: Permissions,
        project_acl: Option<&AccessControlStore>,
    ) -> bool {
        // Check resource's own ACL first
        if let Ok(acl) = parse_acl(doc) {
            if !acl.list.is_empty() {
                return acl.check_permission(principals, required);
            }
        }
        // Fallback to project ACL with scope filtering
        if let Some(proj_acl) = project_acl {
            return proj_acl.check_permission_scoped(
                principals,
                required,
                self.resource_kind_name(),
            );
        }
        false
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

/// Standard `to_external`: renames `_key` → `id`, strips `_id`/`_rev`,
/// and ensures `labels` defaults to `{}` for documents created before the field existed.
pub fn standard_to_external(mut doc: Value) -> Value {
    rename_key_to_id(&mut doc);
    if let Some(obj) = doc.as_object_mut() {
        obj.entry("labels").or_insert_with(|| json!({}));
    }
    doc
}

/// Inject common creation defaults into a document body:
/// labels, annotations (empty if absent), and state audit timestamps.
pub fn inject_create_defaults(body: &mut Value, user_id: &str) {
    let Some(obj) = body.as_object_mut() else {
        return;
    };
    obj.entry("labels").or_insert_with(|| json!({}));
    obj.entry("annotations").or_insert_with(|| json!({}));
    let state = obj.entry("state").or_insert_with(|| json!({}));
    if let Some(state_obj) = state.as_object_mut() {
        state_obj
            .entry("created_at")
            .or_insert_with(|| json!(chrono::Utc::now().to_rfc3339()));
        state_obj
            .entry("created_by")
            .or_insert_with(|| json!(user_id));
        state_obj
            .entry("updated_at")
            .or_insert_with(|| json!(chrono::Utc::now().to_rfc3339()));
    }
}

/// Filter a JSON object to only keep the given field names.
/// Used by `to_list_external` to produce brief representations.
pub fn filter_to_brief(mut value: Value, fields: &[&str]) -> Value {
    if let Some(obj) = value.as_object_mut() {
        obj.retain(|key, _| fields.contains(&key.as_str()));
    }
    value
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

        let scope = entry
            .get("scope")
            .and_then(|v| v.as_str())
            .map(String::from);

        entries.push(AccessControlList {
            permissions,
            principals,
            scope,
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
