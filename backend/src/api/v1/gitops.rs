use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{error::AppError, middleware::auth::AuthenticatedUser, state::AppState};

#[derive(Deserialize)]
pub struct ListQuery {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
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
    let principals = state.db.get_user_principals(&user_id).await?;

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

    let id = body
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::bad_request("missing 'id' field in request body"))?
        .to_string();

    let ctrl = state.controller.for_kind(&kind);

    let godmode = state.has_godmode(&user_id).await.unwrap_or(false);
    let allowed = godmode || ctrl.can_create(&user_id, &body).await?;
    log::debug!("[HANDLER] create_object: can_create={}, godmode={}, user={}, kind={}, id={}", allowed, godmode, user_id, kind, id);
    if !allowed {
        log::debug!("[HANDLER] create_object: DENIED, returning 404");
        return Err(AppError::not_found(format!("{}/{}", kind, id)));
    }

    ctrl.prepare_create(&mut body, &user_id);

    state.db.ensure_collection(&kind).await?;

    let doc = ctrl.to_internal(body, &state.auth)?;
    state
        .db
        .generic_create(&kind, doc)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("unique constraint") || msg.contains("1210") {
                AppError::conflict(format!("{}/{} already exists", kind, id))
            } else {
                AppError::Internal(e)
            }
        })?;

    ctrl.after_create(&id, &user_id, &state.db).await?;

    Ok((axum::http::StatusCode::CREATED, Json(json!({ "id": id }))))
}

/// GET /global/{kind}/{id} — get a single object.
pub async fn get_object(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((kind, id)): Path<(String, String)>,
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
            Ok(Json(ctrl.to_external(d)))
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

    if is_update {
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

    let doc = ctrl.to_internal(body, &state.auth)?;
    state.db.generic_upsert(&kind, &id, doc).await?;

    if is_update {
        ctrl.after_update(&id, &state.db).await?;
    } else {
        ctrl.after_create(&id, &user_id, &state.db).await?;
    }

    Ok(Json(json!({ "id": id })))
}

/// PUT /global/{kind}/{id} — update (fails if not exists).
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

    let godmode = state.has_godmode(&user_id).await.unwrap_or(false);
    if !godmode && !ctrl.can_write(&user_id, Some(&existing)).await? {
        return Err(AppError::not_found(format!("{}/{}", kind, id)));
    }

    let doc = ctrl.to_internal(body, &state.auth)?;
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

    ctrl.after_update(&id, &state.db).await?;

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

    ctrl.after_delete(&id, &state.db).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
