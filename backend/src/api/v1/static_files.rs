//! Unauthenticated static-file endpoint: GET /v1/static/{*path}
//!
//! Serves processed images directly from the object store without requiring
//! authentication. Only two directories are exposed:
//!
//! - `user_avatars/`    — avatar WebP files (HD + thumbnail)
//! - `user_wallpapers/` — wallpaper WebP files (HD + thumbnail)
//!
//! All files in these directories are ULID-named (`{ulid}_hd.webp`,
//! `{ulid}_thumb.webp`), making URLs unguessable in practice.
//!
//! # Caching
//! Responses carry `Cache-Control: public, max-age=31536000, immutable`.
//! Because a new upload always generates a new ULID, cached URLs never
//! become stale — the old path simply stops being referenced.

use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};

use crate::{error::AppError, state::AppState};

/// GET /v1/static/{*path}
pub async fn serve_static(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // Only expose the two public image directories.
    if !path.starts_with("user_avatars/") && !path.starts_with("user_wallpapers/") {
        return Err(AppError::not_found("not found"));
    }

    // Reject path traversal attempts before hitting the object store.
    if path.contains("..") {
        return Err(AppError::not_found("not found"));
    }

    let store = state
        .objectstore
        .as_ref()
        .as_ref()
        .ok_or_else(|| AppError::not_found("not found"))?;

    let data = store
        .get(&path)
        .await
        .map_err(|_| AppError::not_found("not found"))?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/webp")
        .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
        .body(Body::from(data))
        .expect("static response builder is infallible");

    Ok(response)
}
