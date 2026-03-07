//! Media upload endpoint: POST /v1/global/{kind}/{id}/upload/{upload_type}
//!
//! Currently only `kind = "users"` is supported. `upload_type` is one of
//! `"avatar"` or `"wallpaper"`.
//!
//! # Authorization
//! - A user may upload their own media.
//! - `ADM_USER_MANAGER` may upload for any user.
//! - `ADM_GODMODE` bypasses all checks.
//!
//! # Request
//! `Content-Type: multipart/form-data` with a single `"file"` field
//! containing the image bytes (JPEG / PNG / WebP, max 5 MB).
//!
//! # Response
//! `201 Created` with `{ "ulid": "<ulid>" }` — the ULID that was stored in
//! the user document. The image is not yet processed at response time; a
//! background Tokio task continues the conversion.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use chrono::Utc;
use serde_json::json;
use tokio::sync::Semaphore;
use ulid::Ulid;

use crate::{
    error::AppError,
    middleware::auth::AuthenticatedUser,
    services::{
        image_processing::{self, UploadType},
        objectstore::ObjectStoreService,
    },
    state::AppState,
};
use crit_shared::event_models::EventPriority;
use crit_shared::util_models::{PersistentFile, PersistentFileUri, UnprocessedImage, super_permissions};

use super::super::super::services::image_processing::MAX_UPLOAD_BYTES;

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// POST /v1/global/{kind}/{id}/upload/{upload_type}
pub async fn upload_media(
    AuthenticatedUser(caller_id): AuthenticatedUser,
    Path((kind, target_id, upload_type_str)): Path<(String, String, String)>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    log::debug!("[upload] request: caller={caller_id} kind={kind} target={target_id} type={upload_type_str}");

    // Only users are supported for now; projects come later.
    if kind != "users" {
        log::debug!("[upload] rejected: unsupported kind={kind}");
        return Err(AppError::not_found("upload not supported for this resource kind"));
    }

    // Parse upload type.
    let upload_type = match upload_type_str.as_str() {
        "avatar" => UploadType::Avatar,
        "wallpaper" => UploadType::Wallpaper,
        _ => {
            log::debug!("[upload] rejected: invalid upload_type={upload_type_str}");
            return Err(AppError::bad_request(
                "upload_type must be 'avatar' or 'wallpaper'",
            ))
        }
    };

    // Object store must be configured.
    let store = state
        .objectstore
        .as_ref()
        .as_ref()
        .ok_or_else(|| {
            log::error!("[upload] object store not configured");
            AppError::bad_request("object store not configured on this server")
        })?
        .clone();
    log::trace!("[upload] object store available");

    // Pre-flight size check using Content-Length header (if present).
    // Multipart overhead is small; an extra 4 KiB margin is plenty.
    if let Some(cl) = headers.get("content-length") {
        if let Ok(size) = cl.to_str().unwrap_or("0").parse::<usize>() {
            log::trace!("[upload] content-length preflight: {size} bytes (limit {})", MAX_UPLOAD_BYTES);
            if size > MAX_UPLOAD_BYTES + 4096 {
                log::debug!("[upload] rejected: content-length {size} exceeds limit");
                return Err(AppError::bad_request(format!(
                    "request body too large (max {} MB)",
                    MAX_UPLOAD_BYTES / 1024 / 1024
                )));
            }
        }
    }

    // Authorization check.
    log::trace!("[upload] checking upload access for caller={caller_id} target={target_id}");
    check_upload_access(&state, &caller_id, &target_id).await?;
    log::trace!("[upload] access check passed");

    // Verify the target user exists.
    log::trace!("[upload] verifying target user {target_id} exists");
    if state.db.get_user_by_id(&target_id).await?.is_none() {
        log::debug!("[upload] rejected: target user {target_id} not found");
        return Err(AppError::not_found("user not found"));
    }
    log::trace!("[upload] target user {target_id} confirmed");

    // Read the `file` field from the multipart body.
    log::trace!("[upload] reading multipart file field");
    let raw_bytes = read_file_field(&mut multipart).await?;
    log::debug!("[upload] received {} bytes from multipart", raw_bytes.len());

    // Validate format from magic bytes (no I/O until here).
    let fmt = image_processing::detect_format(&raw_bytes)
        .ok_or_else(|| {
            log::debug!("[upload] rejected: unrecognized image format ({} bytes)", raw_bytes.len());
            AppError::bad_request("unsupported image format (accepted: jpeg, png, webp)")
        })?;
    log::debug!("[upload] detected format: {:?}", fmt);

    // Generate ULID and build storage paths.
    let ulid = Ulid::new().to_string().to_lowercase();
    let filename = format!("{}.{}", ulid, fmt.extension());
    let raw_path = format!("raw_uploads/{}", filename);
    log::debug!("[upload] assigned ulid={ulid}, raw_path={raw_path}");

    // Store raw bytes.
    log::trace!("[upload] storing raw bytes to object store: {raw_path}");
    store.put(&raw_path, raw_bytes).await.map_err(|e| {
        log::error!("[upload] failed to store raw upload {raw_path}: {e}");
        AppError::Internal(anyhow::anyhow!("failed to store upload"))
    })?;
    log::debug!("[upload] raw bytes stored: {raw_path}");

    // Record the pending upload.
    log::trace!("[upload] creating unprocessed_images record for {ulid}");
    let unprocessed = serde_json::to_value(UnprocessedImage {
        id: ulid.clone(),
        filename: filename.clone(),
        owner_id: target_id.clone(),
        upload_type: upload_type_str.clone(),
        created_at: Utc::now(),
    })
    .map_err(AppError::from)?;
    state.db.generic_create("unprocessed_images", unprocessed).await?;
    log::debug!("[upload] unprocessed_images record created for {ulid}");

    // Update the user document with this ULID so the field is visible immediately.
    let ulid_field = match upload_type {
        UploadType::Avatar => "avatar_ulid",
        UploadType::Wallpaper => "wallpaper_ulid",
    };
    log::trace!("[upload] patching user {target_id} {ulid_field}={ulid}");
    state
        .db
        .patch_user_image_ulid(&target_id, ulid_field, Some(&ulid))
        .await?;
    log::debug!("[upload] user {target_id} {ulid_field} updated to {ulid}");

    log::info!(
        "[upload] raw upload accepted: {filename} for {target_id} by {caller_id} (bg processing queued)"
    );

    state.events.objectstore_event(EventPriority::Minor, Some(&caller_id), vec![format!("{}/{}", kind, target_id)], Some(serde_json::json!({ "action": "upload", "upload_type": upload_type_str }))).await;

    // Spawn background image processing — response returns immediately.
    // The semaphore ensures only one conversion runs at a time; others queue up.
    let bg_db = state.db.clone();
    let bg_ulid = ulid.clone();
    let bg_sem = state.image_processing_semaphore.clone();
    tokio::spawn(async move {
        process_upload_background(bg_ulid, filename, upload_type, target_id, bg_db, store, bg_sem).await;
    });

    Ok((StatusCode::CREATED, Json(json!({ "ulid": ulid }))))
}

// ---------------------------------------------------------------------------
// Background processing
// ---------------------------------------------------------------------------

async fn process_upload_background(
    ulid: String,
    filename: String,
    upload_type: UploadType,
    owner_id: String,
    db: Arc<crate::db::ArangoDb>,
    store: ObjectStoreService,
    sem: Arc<Semaphore>,
) {
    log::debug!("[upload:bg] task spawned for ulid={ulid} filename={filename} type={upload_type:?}");

    // Acquire the semaphore before doing any CPU-intensive work.
    // If another conversion is already running, this awaits until it finishes.
    // The permit is released automatically when it drops at function exit.
    log::trace!("[upload:bg] waiting for semaphore (ulid={ulid})");
    let _permit = match sem.acquire().await {
        Ok(p) => p,
        Err(_) => {
            log::error!("[upload:bg] semaphore closed, aborting conversion for {ulid}");
            cleanup_raw(&store, &format!("raw_uploads/{}", filename), &db, &ulid).await;
            return;
        }
    };
    log::debug!("[upload:bg] semaphore acquired, starting conversion for {filename}");

    let raw_path = format!("raw_uploads/{}", filename);
    let dir = upload_type.storage_dir();
    let hd_path = format!("{}/{}_hd.webp", dir, ulid);
    let thumb_path = format!("{}/{}_thumb.webp", dir, ulid);
    log::trace!("[upload:bg] paths: raw={raw_path} hd={hd_path} thumb={thumb_path}");

    // Step 1: fetch raw bytes from object storage.
    log::trace!("[upload:bg] step 1: fetching raw bytes from object store: {raw_path}");
    let raw_bytes = match store.get(&raw_path).await {
        Ok(b) => {
            log::debug!("[upload:bg] step 1 OK: fetched {} bytes from {raw_path}", b.len());
            b
        }
        Err(e) => {
            log::error!("[upload:bg] step 1 FAIL: could not fetch raw file {raw_path}: {e}");
            cleanup_raw(&store, &raw_path, &db, &ulid).await;
            return;
        }
    };

    // Step 2: process (crop + resize + WebP encode) on the blocking thread pool.
    // process_image is CPU-intensive (Lanczos3 resize); running it directly in an
    // async task would block Tokio's worker threads and starve the runtime.
    log::debug!("[upload:bg] step 2: spawning blocking image processing for {filename}");
    let raw_bytes_for_processing = raw_bytes.clone();
    let processed = match tokio::task::spawn_blocking(move || {
        image_processing::process_image(&raw_bytes_for_processing, upload_type)
    })
    .await
    {
        Ok(Ok(p)) => {
            log::debug!(
                "[upload:bg] step 2 OK: processed {filename} — hd={} bytes, thumb={} bytes",
                p.hd_size_bytes, p.thumb_size_bytes
            );
            p
        }
        Ok(Err(e)) => {
            log::error!("[upload:bg] step 2 FAIL: image processing failed for {filename}: {e}");
            cleanup_raw(&store, &raw_path, &db, &ulid).await;
            return;
        }
        Err(e) => {
            log::error!("[upload:bg] step 2 FAIL: image processing panicked for {filename}: {e}");
            cleanup_raw(&store, &raw_path, &db, &ulid).await;
            return;
        }
    };

    // Step 3a: store HD variant.
    log::trace!("[upload:bg] step 3a: storing HD image to {hd_path}");
    if let Err(e) = store.put(&hd_path, processed.hd.clone()).await {
        log::error!("[upload:bg] step 3a FAIL: failed to store HD image {hd_path}: {e}");
        cleanup_raw(&store, &raw_path, &db, &ulid).await;
        return;
    }
    log::debug!("[upload:bg] step 3a OK: HD stored at {hd_path}");

    // Step 3b: store thumbnail.
    log::trace!("[upload:bg] step 3b: storing thumbnail to {thumb_path}");
    if let Err(e) = store.put(&thumb_path, processed.thumb.clone()).await {
        log::error!("[upload:bg] step 3b FAIL: failed to store thumbnail {thumb_path}: {e}");
        let _ = store.delete(&hd_path).await;
        cleanup_raw(&store, &raw_path, &db, &ulid).await;
        return;
    }
    log::debug!("[upload:bg] step 3b OK: thumbnail stored at {thumb_path}");

    // Step 4: write persistent file record.
    log::trace!("[upload:bg] step 4: creating persistent_files record for {ulid}");
    let pf = PersistentFile {
        id: ulid.clone(),
        category: dir.to_string(),
        relation_type: "principal".to_string(),
        owner: owner_id.clone(),
        format: "webp".to_string(),
        sizes: vec!["hd".to_string(), "thumb".to_string()],
        total_size_bytes: processed.hd_size_bytes + processed.thumb_size_bytes,
        filenames: vec![hd_path.clone(), thumb_path.clone()],
        uri: PersistentFileUri {
            hd: format!("{}_hd.webp", ulid),
            thumb: format!("{}_thumb.webp", ulid),
        },
        created_at: Utc::now(),
    };

    match serde_json::to_value(&pf) {
        Ok(doc) => {
            if let Err(e) = db.generic_create("persistent_files", doc).await {
                log::error!("[upload:bg] step 4 FAIL: failed to insert persistent_file record for {ulid}: {e}");
                let _ = store.delete(&hd_path).await;
                let _ = store.delete(&thumb_path).await;
                cleanup_raw(&store, &raw_path, &db, &ulid).await;
                return;
            }
            log::debug!("[upload:bg] step 4 OK: persistent_files record created for {ulid}");
        }
        Err(e) => {
            log::error!("[upload:bg] step 4 FAIL: failed to serialize persistent_file for {ulid}: {e}");
            let _ = store.delete(&hd_path).await;
            let _ = store.delete(&thumb_path).await;
            cleanup_raw(&store, &raw_path, &db, &ulid).await;
            return;
        }
    }

    // Step 5: delete raw upload and unprocessed record.
    log::trace!("[upload:bg] step 5: cleaning up raw upload {raw_path}");
    let _ = store.delete(&raw_path).await;
    if let Err(e) = db.generic_delete("unprocessed_images", &ulid).await {
        // Non-fatal — the record will be a stale orphan but the image is live.
        log::warn!("[upload:bg] step 5: could not delete unprocessed_images/{ulid}: {e}");
    } else {
        log::trace!("[upload:bg] step 5 OK: raw and unprocessed_images/{ulid} removed");
    }

    log::info!("[upload:bg] processing complete for {filename} (owner: {owner_id})");
}

/// Delete the raw upload file and the `unprocessed_images` record.
/// Called in all failure paths to avoid orphaned storage.
async fn cleanup_raw(
    store: &ObjectStoreService,
    raw_path: &str,
    db: &crate::db::ArangoDb,
    ulid: &str,
) {
    if let Err(e) = store.delete(raw_path).await {
        log::warn!("[upload:bg] cleanup: could not delete {raw_path}: {e}");
    }
    if let Err(e) = db.generic_delete("unprocessed_images", ulid).await {
        log::warn!("[upload:bg] cleanup: could not delete unprocessed_images/{ulid}: {e}");
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Read the `"file"` multipart field and enforce the size limit.
async fn read_file_field(multipart: &mut Multipart) -> Result<bytes::Bytes, AppError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("multipart parse error: {e}")))?
    {
        if field.name().unwrap_or("") == "file" {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::bad_request(format!("failed to read file field: {e}")))?;

            if data.len() > MAX_UPLOAD_BYTES {
                return Err(AppError::bad_request(format!(
                    "file too large (max {} MB)",
                    MAX_UPLOAD_BYTES / 1024 / 1024
                )));
            }
            return Ok(data);
        }
    }
    Err(AppError::bad_request(
        "missing 'file' field in multipart body",
    ))
}

/// Check whether `caller_id` is allowed to upload media for `target_id`.
///
/// Allowed if:
/// - The caller is the target user themselves.
/// - The caller has `ADM_GODMODE`.
/// - The caller has `ADM_USER_MANAGER`.
///
/// On denial we return 404 (not 403) to avoid leaking whether the target user exists.
async fn check_upload_access(
    state: &AppState,
    caller_id: &str,
    target_id: &str,
) -> Result<(), AppError> {
    if caller_id == target_id {
        return Ok(());
    }
    let is_god = state
        .db
        .has_permission(caller_id, super_permissions::ADM_GODMODE)
        .await?;
    if is_god {
        log::debug!("[upload] godmode bypass for {caller_id} uploading to {target_id}");
        return Ok(());
    }
    let is_user_mgr = state
        .db
        .has_permission(caller_id, super_permissions::ADM_USER_MANAGER)
        .await?;
    if is_user_mgr {
        log::debug!("[upload] ADM_USER_MANAGER bypass for {caller_id} uploading to {target_id}");
        return Ok(());
    }
    log::debug!("[upload] access denied: {caller_id} cannot upload for {target_id}");
    Err(AppError::not_found("user not found"))
}
