use std::sync::Arc;

use axum::{Json, extract::{Path, State}};
use serde_json::{Value, json};

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
