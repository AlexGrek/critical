use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crit_shared::compute_value_hash;

use crate::{error::AppError, middleware::auth::AuthenticatedUser, state::AppState};

#[derive(Deserialize)]
pub struct ListQuery {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub startwith: Option<String>,
}

#[derive(Deserialize)]
pub struct GetObjectQuery {
    pub with_history: Option<String>,
}

/// Validate that a kind string is a safe collection name (alphanumeric + underscores).
pub fn validate_kind(kind: &str) -> Result<(), AppError> {
    if kind.is_empty() {
        return Err(AppError::bad_request("kind must not be empty"));
    }
    if !kind
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(AppError::bad_request(
            "kind must contain only alphanumeric characters and underscores",
        ));
    }
    Ok(())
}

/// GET /global/{kind} — list all objects of this kind.
/// Supports optional pagination via `?limit=N&cursor=<key>`.
/// ACL filtering is pushed into a single AQL query for efficiency.
pub async fn list_objects(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(kind): Path<String>,
    Query(query): Query<ListQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;
    state.db.ensure_collection(&kind).await?;

    let ctrl = state.controller.for_kind(&kind);

    // Godmode bypasses all ACL checks
    let godmode = state.has_godmode(&user_id).await.unwrap_or(false);

    // Resolve principals once for the entire request
    let principals = state.get_cached_principals(&user_id).await?;

    // Check super-permission bypass. If None (no super-permission defined),
    // treat as fully permissive (matches DefaultKindController behavior).
    let super_bypass = godmode || match ctrl.super_permission() {
        Some(perm) => state
            .db
            .has_permission_with_principals(&principals, perm)
            .await?,
        None => true,
    };

    let result = state
        .db
        .generic_list_acl(
            &kind,
            &principals,
            ctrl.read_permission_bits(),
            super_bypass,
            ctrl.list_projection_fields(),
            query.limit,
            query.cursor.as_deref(),
        )
        .await?;

    let filtered: Vec<Value> = result
        .docs
        .into_iter()
        .map(|doc| ctrl.to_list_external(doc))
        .collect();

    if query.limit.is_some() {
        let mut response = json!({
            "items": filtered,
            "has_more": result.has_more,
        });
        if let Some(cursor) = result.next_cursor {
            response["next_cursor"] = Value::String(cursor);
        }
        Ok(Json(response))
    } else {
        Ok(Json(json!({ "items": filtered })))
    }
}

/// POST /global/{kind} — create a new object (id read from body).
pub async fn create_object(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(kind): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(mut body): Json<Value>,
) -> Result<impl IntoResponse, AppError> {
    log::debug!("[HANDLER] create_object: user={}, kind={}", user_id, kind);
    validate_kind(&kind)?;

    // Read the raw id for the auth check — to_internal hasn't run yet so the
    // id may not have its kind prefix (e.g. "qqq" before becoming "g_qqq").
    let raw_id = body
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::bad_request("missing 'id' field in request body"))?
        .to_string();

    let ctrl = state.controller.for_kind(&kind);

    let godmode = state.has_godmode(&user_id).await.unwrap_or(false);
    let allowed = godmode || ctrl.can_create(&user_id, &body).await?;
    log::debug!("[HANDLER] create_object: can_create={}, godmode={}, user={}, kind={}, raw_id={}", allowed, godmode, user_id, kind, raw_id);
    if !allowed {
        log::debug!("[HANDLER] create_object: DENIED for user={}, kind={}, id={}", user_id, kind, raw_id);
        return Err(AppError::forbidden(format!("not allowed to create {}/{}", kind, raw_id)));
    }

    ctrl.prepare_create(&mut body, &user_id);

    state.db.ensure_collection(&kind).await?;

    // to_internal may transform the id (e.g. add a kind prefix, rename to _key).
    // Extract the final _key from the transformed document so that after_create,
    // error messages, and the success response all use the canonical stored key.
    let mut doc = ctrl.to_internal(body, &state.auth)?;
    // Compute and inject the desired-state hash before writing to DB.
    let hash = compute_value_hash(&doc);
    if let Some(obj) = doc.as_object_mut() {
        obj.insert("hash_code".to_string(), json!(hash));
    }
    let final_id = doc
        .get("_key")
        .and_then(|v| v.as_str())
        .unwrap_or(&raw_id)
        .to_string();

    // Validate ACL principals (e.g. group members check) before writing
    ctrl.validate_acl_principals(&doc, &state.db).await?;

    state
        .db
        .generic_create(&kind, doc)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("unique constraint") || msg.contains("1210") {
                AppError::conflict(format!("{}/{} already exists", kind, final_id))
            } else {
                AppError::Internal(e)
            }
        })?;

    if let Err(e) = ctrl.after_create(&final_id, &user_id, &state.db).await {
        log::error!("[HANDLER] create_object: after_create hook failed: kind={}, id={}, error={}", kind, final_id, e);
        return Err(e);
    }

    // Write initial history entry — non-fatal
    if let Ok(Some(snap)) = state.db.generic_get(&kind, &final_id).await {
        if let Err(e) = state.db.write_history_entry(&kind, &final_id, snap, &user_id).await {
            log::error!("[HANDLER] create_object: write_history_entry failed: kind={}, id={}, error={}", kind, final_id, e);
        }
    }

    Ok((axum::http::StatusCode::CREATED, Json(json!({ "id": final_id }))))
}

/// GET /global/{kind}/{id} — get a single object.
/// 404 if not found or if ACL check fails, to avoid leaking existence information.
/// Supports `?with_history=true` to attach the latest history revision as `_history`.
pub async fn get_object(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((kind, id)): Path<(String, String)>,
    Query(params): Query<GetObjectQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;

    let ctrl = state.controller.for_kind(&kind);
    let doc = state.db.generic_get(&kind, &id).await?;
    match doc {
        Some(d) => {
            let godmode = state.has_godmode(&user_id).await.unwrap_or(false);
            if !godmode && !ctrl.can_read(&user_id, Some(&d)).await? {
                return Err(AppError::not_found(format!("{}/{}", kind, id)));
            }
            let mut result = ctrl.to_external(d);
            if params.with_history.as_deref() == Some("true") {
                if let Ok(Some(history)) = state.db.get_latest_history_entry(&kind, &id).await {
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert("_history".to_string(), history);
                    }
                }
            }
            Ok(Json(result))
        }
        None => Err(AppError::not_found(format!("{}/{}", kind, id))),
    }
}

/// POST /global/{kind}/{id} — upsert (create or replace).
pub async fn upsert_object(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((kind, id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    Json(mut body): Json<Value>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;

    if let Some(obj) = body.as_object_mut() {
        obj.insert("id".to_string(), Value::String(id.clone()));
    }

    let ctrl = state.controller.for_kind(&kind);
    let existing = state.db.generic_get(&kind, &id).await?;
    let is_update = existing.is_some();

    let godmode = state.has_godmode(&user_id).await.unwrap_or(false);

    // Extract client hash before `to_internal` consumes `body`.
    let client_hash = body
        .get("hash_code")
        .and_then(|v| v.as_str())
        .map(String::from);

    if is_update {
        // Validate hash if client sent one — prevent lost updates.
        if let Some(ref ch) = client_hash {
            let server_hash = existing
                .as_ref()
                .and_then(|d| d.get("hash_code"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !server_hash.is_empty() && ch != server_hash {
                return Err(AppError::conflict(format!(
                    "{}/{} was modified since last read (expected hash {}, server has {})",
                    kind, id, ch, server_hash
                )));
            }
        }
        if !godmode && !ctrl.can_write(&user_id, existing.as_ref()).await? {
            return Err(AppError::not_found(format!("{}/{}", kind, id)));
        }
    } else {
        if !godmode && !ctrl.can_create(&user_id, &body).await? {
            return Err(AppError::not_found(format!("{}/{}", kind, id)));
        }
        ctrl.prepare_create(&mut body, &user_id);
    }

    state.db.ensure_collection(&kind).await?;

    let mut doc = ctrl.to_internal(body, &state.auth)?;
    // Compute and inject the desired-state hash before writing to DB.
    let hash = compute_value_hash(&doc);
    if let Some(obj) = doc.as_object_mut() {
        obj.insert("hash_code".to_string(), json!(hash));
    }

    // Validate ACL principals (e.g. group members check) before writing
    ctrl.validate_acl_principals(&doc, &state.db).await?;

    state.db.generic_upsert(&kind, &id, doc).await?;

    if is_update {
        if let Err(e) = ctrl.after_update(&id, &state.db).await {
            log::error!("[HANDLER] upsert_object: after_update hook failed: kind={}, id={}, error={}", kind, id, e);
            return Err(e);
        }
    } else {
        if let Err(e) = ctrl.after_create(&id, &user_id, &state.db).await {
            log::error!("[HANDLER] upsert_object: after_create hook failed: kind={}, id={}, error={}", kind, id, e);
            return Err(e);
        }
    }

    // Write history entry after upsert — non-fatal
    if let Ok(Some(snap)) = state.db.generic_get(&kind, &id).await {
        if let Err(e) = state.db.write_history_entry(&kind, &id, snap, &user_id).await {
            log::error!("[HANDLER] upsert_object: write_history_entry failed: kind={}, id={}, error={}", kind, id, e);
        }
    }

    Ok(Json(json!({ "id": id })))
}

/// PUT /global/{kind}/{id} — update (fails if not exists with 404 or on update conflict with 409).
/// TODO: ensure it does so
pub async fn update_object(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((kind, id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    Json(mut body): Json<Value>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;

    if let Some(obj) = body.as_object_mut() {
        obj.insert("id".to_string(), Value::String(id.clone()));
    }

    let ctrl = state.controller.for_kind(&kind);
    let existing = state.db.generic_get(&kind, &id).await?;
    let existing = existing.ok_or_else(|| AppError::not_found(format!("{}/{}", kind, id)))?;

    // Extract client hash before `to_internal` consumes `body`.
    let client_hash = body
        .get("hash_code")
        .and_then(|v| v.as_str())
        .map(String::from);

    // Validate hash if client sent one — prevent lost updates.
    if let Some(ref ch) = client_hash {
        let server_hash = existing
            .get("hash_code")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !server_hash.is_empty() && ch != server_hash {
            return Err(AppError::conflict(format!(
                "{}/{} was modified since last read (expected hash {}, server has {})",
                kind, id, ch, server_hash
            )));
        }
    }

    let godmode = state.has_godmode(&user_id).await.unwrap_or(false);
    if !godmode && !ctrl.can_write(&user_id, Some(&existing)).await? {
        return Err(AppError::not_found(format!("{}/{}", kind, id)));
    }

    let mut doc = ctrl.to_internal(body, &state.auth)?;
    // Compute and inject the desired-state hash before writing to DB.
    let hash = compute_value_hash(&doc);
    if let Some(obj) = doc.as_object_mut() {
        obj.insert("hash_code".to_string(), json!(hash));
    }

    // Validate ACL principals (e.g. group members check) before writing
    ctrl.validate_acl_principals(&doc, &state.db).await?;

    state
        .db
        .generic_update(&kind, &id, doc)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("document not found") {
                AppError::not_found(format!("{}/{}", kind, id))
            } else {
                AppError::Internal(e)
            }
        })?;

    if let Err(e) = ctrl.after_update(&id, &state.db).await {
        log::error!("[HANDLER] update_object: after_update hook failed: kind={}, id={}, error={}", kind, id, e);
        return Err(e);
    }

    // Write history entry on update — non-fatal
    if let Ok(Some(snap)) = state.db.generic_get(&kind, &id).await {
        if let Err(e) = state.db.write_history_entry(&kind, &id, snap, &user_id).await {
            log::error!("[HANDLER] update_object: write_history_entry failed: kind={}, id={}, error={}", kind, id, e);
        }
    }

    Ok(Json(json!({ "id": id })))
}

/// DELETE /global/{kind}/{id} — delete an object.
pub async fn delete_object(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((kind, id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;

    let ctrl = state.controller.for_kind(&kind);
    let existing = state.db.generic_get(&kind, &id).await?;
    let existing = existing.ok_or_else(|| AppError::not_found(format!("{}/{}", kind, id)))?;

    let godmode = state.has_godmode(&user_id).await.unwrap_or(false);
    if !godmode && !ctrl.can_write(&user_id, Some(&existing)).await? {
        return Err(AppError::not_found(format!("{}/{}", kind, id)));
    }

    state
        .db
        .generic_soft_delete(&kind, &id, &user_id)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found or already deleted") {
                AppError::not_found(format!("{}/{}", kind, id))
            } else {
                AppError::Internal(e)
            }
        })?;

    if let Err(e) = ctrl.after_delete(&id, &state.db).await {
        log::error!("[HANDLER] delete_object: after_delete hook failed: kind={}, id={}, error={}", kind, id, e);
        return Err(e);
    }

    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// GET /global/{kind}/search?startwith={prefix} — quick prefix search on _key.
/// Returns up to 15 brief results. ACL-filtered the same way as the list endpoint.
pub async fn search_objects(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(kind): Path<String>,
    Query(query): Query<SearchQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;
    state.db.ensure_collection(&kind).await?;

    let startwith = query.startwith.as_deref().unwrap_or("");

    let ctrl = state.controller.for_kind(&kind);

    let godmode = state.has_godmode(&user_id).await.unwrap_or(false);
    let principals = state.get_cached_principals(&user_id).await?;

    let super_bypass = godmode || match ctrl.super_permission() {
        Some(perm) => state
            .db
            .has_permission_with_principals(&principals, perm)
            .await?,
        None => true,
    };

    let docs = state
        .db
        .generic_search_acl(
            &kind,
            &principals,
            ctrl.read_permission_bits(),
            super_bypass,
            ctrl.list_projection_fields(),
            startwith,
        )
        .await?;

    let items: Vec<Value> = docs
        .into_iter()
        .map(|doc| ctrl.to_list_external(doc))
        .collect();

    Ok(Json(json!({ "items": items })))
}
