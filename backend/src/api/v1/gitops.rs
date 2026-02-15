use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use serde_json::{Value, json};

use crate::{error::AppError, middleware::auth::Auth, state::AppState};

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

/// Convert an internal ArangoDB document to the external gitops representation.
/// Renames `_key` → `id`, strips ArangoDB internal fields and sensitive data.
fn to_external(kind: &str, mut doc: Value) -> Value {
    if let Some(obj) = doc.as_object_mut() {
        // Rename _key → id
        if let Some(key) = obj.remove("_key") {
            obj.insert("id".to_string(), key);
        }
        // Strip ArangoDB internal fields
        obj.remove("_id");
        obj.remove("_rev");
        // Strip sensitive fields for users
        if kind == "users" {
            obj.remove("password_hash");
        }
    }
    doc
}

/// Convert an external gitops request body to an internal ArangoDB document.
/// Renames `id` → `_key`, hashes password for users.
fn to_internal(kind: &str, mut body: Value, auth: &Auth) -> Result<Value, AppError> {
    if let Some(obj) = body.as_object_mut() {
        // Rename id → _key
        if let Some(id) = obj.remove("id") {
            obj.insert("_key".to_string(), id);
        }
        // Hash password for users
        if kind == "users" {
            if let Some(password) = obj.remove("password") {
                if let Some(pw_str) = password.as_str() {
                    let hash = auth.hash_password(pw_str)?;
                    obj.insert("password_hash".to_string(), Value::String(hash));
                }
            }
        }
    }
    Ok(body)
}

/// GET /global/{kind} — list all objects of this kind.
pub async fn list_objects(
    Path(kind): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;
    state.db.ensure_collection(&kind).await?;

    let docs = state.db.generic_list(&kind).await?;
    let external: Vec<Value> = docs.into_iter().map(|d| to_external(&kind, d)).collect();
    Ok(Json(json!({ "items": external })))
}

/// POST /global/{kind} — create a new object (id read from body).
pub async fn create_object(
    Path(kind): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;

    let id = body
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::bad_request("missing 'id' field in request body"))?
        .to_string();

    state.db.ensure_collection(&kind).await?;

    let doc = to_internal(&kind, body, &state.auth)?;
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
    Path((kind, id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;

    let doc = state.db.generic_get(&kind, &id).await?;
    match doc {
        Some(d) => Ok(Json(to_external(&kind, d))),
        None => Err(AppError::not_found(format!("{}/{}", kind, id))),
    }
}

/// POST /global/{kind}/{id} — upsert (create or replace).
pub async fn upsert_object(
    Path((kind, id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    Json(mut body): Json<Value>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;

    // Ensure id in body matches path
    if let Some(obj) = body.as_object_mut() {
        obj.insert("id".to_string(), Value::String(id.clone()));
    }

    state.db.ensure_collection(&kind).await?;

    let doc = to_internal(&kind, body, &state.auth)?;
    state.db.generic_upsert(&kind, &id, doc).await?;

    Ok(Json(json!({ "id": id })))
}

/// PUT /global/{kind}/{id} — update (fails if not exists).
pub async fn update_object(
    Path((kind, id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    Json(mut body): Json<Value>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;

    // Ensure id in body matches path
    if let Some(obj) = body.as_object_mut() {
        obj.insert("id".to_string(), Value::String(id.clone()));
    }

    let doc = to_internal(&kind, body, &state.auth)?;
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
    Path((kind, id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;

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
