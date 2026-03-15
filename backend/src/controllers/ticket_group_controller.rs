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

    fn to_internal(&self, mut body: Value, _auth: &Auth) -> Result<Value, AppError> {
        // Validate and normalise the `id` field (2-6 capital letters, add tg_ prefix)
        if let Some(obj) = body.as_object_mut() {
            if let Some(id_val) = obj.get("id").and_then(|v| v.as_str()) {
                let code = validate_ticket_group_id(id_val)
                    .map_err(|e| AppError::Validation(e))?;
                let prefixed = format!("tg_{}", code);
                obj.insert("id".to_string(), serde_json::Value::String(prefixed));
            }
        }

        strip_unknown_fields(&mut body, TICKET_GROUP_ALLOWED_FIELDS);
        Ok(standard_to_internal(body))
    }

    fn to_external(&self, doc: Value) -> Value {
        standard_to_external(doc)
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
