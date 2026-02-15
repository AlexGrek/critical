use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use serde_json::{Value, json};

use crate::{controllers::gitops_controller::GitopsController, error::AppError, middleware::auth::AuthenticatedUser, state::AppState};

/// Validate that a kind string is a safe collection name (alphanumeric + underscores).
fn validate_kind(kind: &str) -> Result<(), AppError> {
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
pub async fn list_objects(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(kind): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;
    state.db.ensure_collection(&kind).await?;

    let gitops = &state.controller.gitops;
    let docs = state.db.generic_list(&kind).await?;

    let mut filtered = Vec::new();
    for doc in docs {
        if gitops.can_read(&user_id, &kind, Some(&doc)).await? {
            filtered.push(GitopsController::to_external(&kind, doc));
        }
    }

    Ok(Json(json!({ "items": filtered })))
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

    let gitops = &state.controller.gitops;

    let allowed = gitops.can_write(&user_id, &kind, None).await?;
    log::debug!("[HANDLER] create_object: can_write={}, user={}, kind={}, id={}", allowed, user_id, kind, id);
    if !allowed {
        log::debug!("[HANDLER] create_object: DENIED, returning 404");
        return Err(AppError::not_found(format!("{}/{}", kind, id)));
    }

    if kind == "projects" {
        GitopsController::ensure_creator_in_acl(&mut body, &user_id);
    }

    state.db.ensure_collection(&kind).await?;

    let doc = GitopsController::to_internal(&kind, body, &state.auth)?;
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

    Ok((axum::http::StatusCode::CREATED, Json(json!({ "id": id }))))
}

/// GET /global/{kind}/{id} — get a single object.
pub async fn get_object(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((kind, id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;

    let gitops = &state.controller.gitops;
    let doc = state.db.generic_get(&kind, &id).await?;
    match doc {
        Some(d) => {
            if !gitops.can_read(&user_id, &kind, Some(&d)).await? {
                return Err(AppError::not_found(format!("{}/{}", kind, id)));
            }
            Ok(Json(GitopsController::to_external(&kind, d)))
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

    let gitops = &state.controller.gitops;
    let existing = state.db.generic_get(&kind, &id).await?;

    if !gitops.can_write(&user_id, &kind, existing.as_ref()).await? {
        return Err(AppError::not_found(format!("{}/{}", kind, id)));
    }

    if kind == "projects" && existing.is_none() {
        GitopsController::ensure_creator_in_acl(&mut body, &user_id);
    }

    state.db.ensure_collection(&kind).await?;

    let doc = GitopsController::to_internal(&kind, body, &state.auth)?;
    state.db.generic_upsert(&kind, &id, doc).await?;

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

    let gitops = &state.controller.gitops;
    let existing = state.db.generic_get(&kind, &id).await?;
    let existing = existing.ok_or_else(|| AppError::not_found(format!("{}/{}", kind, id)))?;

    if !gitops.can_write(&user_id, &kind, Some(&existing)).await? {
        return Err(AppError::not_found(format!("{}/{}", kind, id)));
    }

    let doc = GitopsController::to_internal(&kind, body, &state.auth)?;
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

    Ok(Json(json!({ "id": id })))
}

/// DELETE /global/{kind}/{id} — delete an object.
pub async fn delete_object(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((kind, id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;

    let gitops = &state.controller.gitops;
    let existing = state.db.generic_get(&kind, &id).await?;
    let existing = existing.ok_or_else(|| AppError::not_found(format!("{}/{}", kind, id)))?;

    if !gitops.can_write(&user_id, &kind, Some(&existing)).await? {
        return Err(AppError::not_found(format!("{}/{}", kind, id)));
    }

    state
        .db
        .generic_delete(&kind, &id)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("document not found") {
                AppError::not_found(format!("{}/{}", kind, id))
            } else {
                AppError::Internal(e)
            }
        })?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
