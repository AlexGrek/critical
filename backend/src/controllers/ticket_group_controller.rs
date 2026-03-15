use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::db::ArangoDb;
use crate::error::AppError;
use crate::middleware::auth::Auth;

use super::gitops_controller::{
    KindController, inject_create_defaults, standard_to_external, standard_to_internal,
    strip_unknown_fields,
};

/// Top-level fields accepted by the ticket group API.
/// Any field not in this list is stripped in `to_internal` before the DB write.
const TICKET_GROUP_ALLOWED_FIELDS: &[&str] = &[
    "id", "_key",
    "name", "description", "ticket_types", "project",
    "labels", "annotations", "acl", "state", "deletion", "hash_code",
];

/// Fields returned in list queries (DB-level projection — avoids fetching ticket_types).
const TICKET_GROUP_LIST_FIELDS: &[&str] = &[
    "_key", "name", "description", "project", "acl", "labels", "annotations", "state",
    "deletion", "hash_code",
];

pub struct TicketGroupController {
    pub db: Arc<ArangoDb>,
}

impl TicketGroupController {
    pub fn new(db: Arc<ArangoDb>) -> Self {
        Self { db }
    }
}

/// Validate a raw ticket group id (with or without the `tg_` prefix).
///
/// Returns the bare code (no prefix) on success, or a descriptive error string.
fn validate_ticket_group_id(raw: &str) -> Result<&str, String> {
    let code = raw.strip_prefix("tg_").unwrap_or(raw);
    if code.len() < 2 || code.len() > 6 {
        return Err(format!(
            "ticket group id must be 2-6 capital letters, got '{}' ({} chars)",
            code,
            code.len()
        ));
    }
    if !code.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(format!(
            "ticket group id must be uppercase letters A-Z only, got '{}'",
            code
        ));
    }
    Ok(code)
}

/// Extract the bare code from a user-supplied id (strips `tg_` prefix if present).
/// Does NOT validate — call `validate_ticket_group_id` first.
fn bare_code(id: &str) -> &str {
    id.strip_prefix("tg_").unwrap_or(id)
}

/// Build the composite ArangoDB `_key` for a ticket group.
/// Format: `{project}__{code}` where code is the bare uppercase letters.
fn composite_key(project_id: &str, code: &str) -> String {
    format!("{}__{}", project_id, code)
}

#[async_trait]
impl KindController for TicketGroupController {
    /// Scoped resources use hybrid ACL (resource ACL → project ACL fallback).
    /// `can_read` / `can_write` are not called by `scoped_gitops` handlers, but
    /// are required by the trait.  We return true here and let the scoped handler
    /// enforce project-level permissions via `check_hybrid_acl`.
    async fn can_read(&self, _user_id: &str, _doc: Option<&Value>) -> Result<bool, AppError> {
        Ok(true)
    }

    async fn can_write(&self, _user_id: &str, _doc: Option<&Value>) -> Result<bool, AppError> {
        Ok(true)
    }

    /// Compute the globally-unique ArangoDB `_key` for a scoped ticket group.
    ///
    /// Ticket group codes (e.g. `BUG`, `FEAT`) are only unique within a project,
    /// not across projects. To avoid `_key` collisions in the flat `ticketgroups`
    /// collection, we use a composite key `{project}__{code}`.
    fn scoped_db_key(&self, project_id: &str, id: &str) -> String {
        // id may be "BUG", "tg_BUG", or even the full composite key on re-entry — normalise
        let code = bare_code(id);
        // Strip composite prefix if the caller already passed project__code
        let code = code.strip_prefix(&format!("{project_id}__")).unwrap_or(code);
        composite_key(project_id, code)
    }

    fn to_internal(&self, mut body: Value, _auth: &Auth) -> Result<Value, AppError> {
        // Validate and normalise the `id` field; set composite `_key`.
        if let Some(obj) = body.as_object_mut() {
            let project_id = obj
                .get("project")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if let Some(id_val) = obj.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()) {
                let code = validate_ticket_group_id(&id_val)
                    .map_err(|e| AppError::Validation(e))?;

                if !project_id.is_empty() {
                    // Set composite _key for global uniqueness and remove `id` so that
                    // standard_to_internal's rename_id_to_key does not overwrite _key.
                    obj.insert(
                        "_key".to_string(),
                        serde_json::Value::String(composite_key(&project_id, code)),
                    );
                    obj.remove("id");
                } else {
                    // No project yet (e.g. internal calls without project context):
                    // fall back to simple tg_{code} key.
                    obj.insert("id".to_string(), serde_json::Value::String(format!("tg_{}", code)));
                }
            }
        }

        strip_unknown_fields(&mut body, TICKET_GROUP_ALLOWED_FIELDS);
        // standard_to_internal renames id → _key; since we removed id above (when we
        // have a composite key), this is a no-op and _key is already set correctly.
        Ok(standard_to_internal(body))
    }

    fn to_external(&self, doc: Value) -> Value {
        // Recover the user-visible `id = "tg_{code}"` from the composite `_key` BEFORE
        // standard_to_external renames _key → id (which would expose the raw composite key).
        let tg_id = doc
            .get("_key")
            .and_then(|v| v.as_str())
            .and_then(|key| key.splitn(2, "__").nth(1))
            .map(|code| format!("tg_{}", code));

        let mut out = standard_to_external(doc);

        // Override the `id` field with the user-visible form if we extracted a code.
        if let Some(id) = tg_id {
            if let Some(obj) = out.as_object_mut() {
                obj.insert("id".to_string(), serde_json::Value::String(id));
            }
        }
        out
    }

    fn to_list_external(&self, doc: Value) -> Value {
        // ticket_types are excluded at the DB projection level; just do standard transform.
        self.to_external(doc)
    }

    fn list_projection_fields(&self) -> Option<&'static [&'static str]> {
        Some(TICKET_GROUP_LIST_FIELDS)
    }

    fn is_scoped(&self) -> bool {
        true
    }

    /// No super-permission short-circuit: access is governed entirely by the
    /// parent project's ACL (with hybrid fallback to the resource's own ACL).
    fn super_permission(&self) -> Option<&str> {
        None
    }

    fn prepare_create(&self, body: &mut Value, user_id: &str) {
        log::debug!(
            "[ACL] TicketGroupController::prepare_create: user={}",
            user_id
        );
        inject_create_defaults(body, user_id);
        // No custom ACL entry — access inherits from the project ACL.
    }
}
