use std::collections::HashMap;
use std::sync::Arc;

use axum::{Json, extract::{Path, Query, State}};
use serde_json::{Value, json};

use crit_shared::util_models::{AccessControlStore, Permissions};

use crate::{error::AppError, state::AppState};

/// List all non-system ArangoDB collections in the current database.
///
/// `GET /v1/debug/collections`
/// Requires ADM_GODMODE (enforced by `godmode_middleware` on the route group).
pub async fn list_collections(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let collections = state
        .db
        .list_collections()
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(json!({ "collections": collections })))
}

/// Dump all raw documents from a named collection.
///
/// `GET /v1/debug/collections/{name}`
/// Requires ADM_GODMODE (enforced by `godmode_middleware` on the route group).
/// Returns `{ "collection": "<name>", "count": N, "documents": [...] }`.
pub async fn get_collection_data(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Value>, AppError> {
    let docs = state
        .db
        .dump_collection(&name)
        .await
        .map_err(|e| {
            if e.to_string().contains("system collections") {
                AppError::BadRequest(e.to_string())
            } else {
                AppError::Internal(e)
            }
        })?;
    let count = docs.len();
    Ok(Json(json!({
        "collection": name,
        "count": count,
        "documents": docs,
    })))
}

/// Inspect access for a user against an optional resource and permission bits.
///
/// `GET /v1/debug/access?user={user_id}[&resource={kind}/{id}][&permission={perm_bits}]`
/// Requires ADM_GODMODE (enforced by `godmode_middleware` on the route group).
///
/// Returns a JSON report with:
/// - user's resolved principals (direct + transitive groups)
/// - user's super-permissions
/// - resource's raw ACL entries (if `resource` is provided)
/// - ACL entries that match the user's principals
/// - access decision and reason (if `permission` bits are provided)
pub async fn get_access_debug(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, AppError> {
    let user_id = params
        .get("user")
        .ok_or_else(|| AppError::BadRequest("missing 'user' query param".to_string()))?;

    let principals = state
        .get_cached_principals(user_id)
        .await
        .map_err(AppError::Internal)?;

    let super_permissions = state
        .db
        .get_user_permissions(user_id)
        .await
        .map_err(AppError::Internal)?;

    let mut report = json!({
        "user": user_id,
        "principals": principals,
        "super_permissions": super_permissions,
    });

    if let Some(resource) = params.get("resource") {
        let parts: Vec<&str> = resource.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Err(AppError::BadRequest(
                "'resource' must be in format 'kind/id'".to_string(),
            ));
        }
        let (kind, id) = (parts[0], parts[1]);

        let doc = state
            .db
            .generic_get(kind, id)
            .await
            .map_err(AppError::Internal)?;

        let obj = report.as_object_mut().unwrap();

        match doc {
            None => {
                obj.insert("resource_found".to_string(), json!(false));
            }
            Some(doc) => {
                let acl_raw = doc.get("acl").cloned().unwrap_or(json!(null));
                let acl: AccessControlStore = doc
                    .get("acl")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                // Find ACL entries that overlap with the user's principals.
                let matching_entries: Vec<Value> = acl
                    .list
                    .iter()
                    .enumerate()
                    .filter_map(|(i, entry)| {
                        let matched: Vec<&String> = entry
                            .principals
                            .iter()
                            .filter(|p| principals.contains(p))
                            .collect();
                        if matched.is_empty() {
                            return None;
                        }
                        Some(json!({
                            "entry_index": i,
                            "permission_bits": entry.permissions.bits(),
                            "matching_principals": matched,
                        }))
                    })
                    .collect();

                obj.insert("resource_found".to_string(), json!(true));
                obj.insert("resource_acl".to_string(), acl_raw);
                obj.insert("matching_acl_entries".to_string(), json!(matching_entries));

                if let Some(perm_str) = params.get("permission") {
                    let perm_bits: u8 = perm_str.parse().map_err(|_| {
                        AppError::BadRequest(
                            "'permission' must be a numeric u8 value".to_string(),
                        )
                    })?;
                    let required = Permissions::from_bits_truncate(perm_bits);

                    let is_godmode = super_permissions
                        .iter()
                        .any(|p| p == "adm_godmode");
                    let has_acl_access = acl.check_permission(&principals, required);
                    let granted = is_godmode || has_acl_access;
                    let reason = if is_godmode {
                        "godmode bypass"
                    } else if has_acl_access {
                        "ACL match"
                    } else {
                        "denied"
                    };

                    obj.insert(
                        "access_result".to_string(),
                        json!({
                            "required_bits": perm_bits,
                            "granted": granted,
                            "reason": reason,
                        }),
                    );
                }
            }
        }
    }

    Ok(Json(report))
}

/// List system events with optional kind/priority filters and pagination.
///
/// `GET /v1/debug/events`
/// Query params:
///   - `kind` — filter by EventKind variant name (e.g. "EntityManagement", "Server", "Error")
///   - `priority` — filter by EventPriority variant name (e.g. "Lifecycle", "Important", "Note")
///   - `limit` — max docs per page (default 50, max 200)
///   - `cursor` — keyset pagination cursor (value of `_key` from previous response's `next_cursor`)
///
/// Requires ADM_GODMODE (enforced by godmode_middleware on the route group).
/// Returns `{ "events": [...], "has_more": bool, "next_cursor": string|null, "count": N }`
pub async fn list_events(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, AppError> {
    let kind = params.get("kind").cloned();
    let priority = params.get("priority").cloned();
    let limit = params
        .get("limit")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(50)
        .min(200);
    let cursor = params.get("cursor").cloned();

    let result = state
        .db
        .list_system_events(kind.as_deref(), priority.as_deref(), limit, cursor)
        .await
        .map_err(AppError::Internal)?;

    let count = result.docs.len();
    Ok(Json(json!({
        "events": result.docs,
        "has_more": result.has_more,
        "next_cursor": result.next_cursor,
        "count": count,
    })))
}
