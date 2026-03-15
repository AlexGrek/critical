use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde_json::{Value, json};

use crate::{
    controllers::gitops_controller::parse_acl,
    error::AppError,
    middleware::auth::AuthenticatedUser,
    state::AppState,
};
use crit_shared::compute_value_hash;
use crit_shared::event_models::EventPriority;
use crit_shared::util_models::Permissions;

use super::gitops::{ListQuery, validate_kind};

/// Validate that a project exists and is not deleted. Returns the project doc.
async fn validate_project(state: &AppState, project_id: &str) -> Result<Value, AppError> {
    let project = state.db.generic_get("projects", project_id).await?;
    project.ok_or_else(|| AppError::not_found(format!("projects/{}", project_id)))
}

/// Resolve user principals and check super-permission bypass for a controller.
/// Also checks godmode — if the user has ADM_GODMODE, super_bypass is always true.
async fn resolve_auth(
    state: &AppState,
    user_id: &str,
    super_perm: Option<&str>,
) -> Result<(Vec<String>, bool), AppError> {
    let godmode = state.has_godmode(user_id).await.unwrap_or(false);
    let principals = state.get_cached_principals(user_id).await?;
    let super_bypass = godmode || match super_perm {
        Some(perm) => state
            .db
            .has_permission_with_principals(&principals, perm)
            .await?,
        None => false,
    };
    Ok((principals, super_bypass))
}

/// GET /v1/projects/{project}/{kind}
pub async fn list_scoped_objects(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((project_id, kind)): Path<(String, String)>,
    Query(query): Query<ListQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;
    let _project_doc = validate_project(&state, &project_id).await?;

    let ctrl = state.controller.for_kind(&kind);

    if !ctrl.is_scoped() {
        return Err(AppError::bad_request(format!(
            "'{}' is not a project-scoped resource kind",
            kind
        )));
    }

    state.db.ensure_collection(&kind).await?;

    let (principals, super_bypass) =
        resolve_auth(&state, &user_id, ctrl.super_permission()).await?;

    let result = state
        .db
        .generic_list_scoped(
            &kind,
            &project_id,
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

/// GET /v1/projects/{project}/{kind}/{id}
pub async fn get_scoped_object(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((project_id, kind, id)): Path<(String, String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;
    let project_doc = validate_project(&state, &project_id).await?;

    let ctrl = state.controller.for_kind(&kind);
    if !ctrl.is_scoped() {
        return Err(AppError::bad_request(format!(
            "'{}' is not a project-scoped resource kind",
            kind
        )));
    }

    let db_key = ctrl.scoped_db_key(&project_id, &id);
    let doc = state
        .db
        .generic_get_scoped(&kind, &project_id, &db_key)
        .await?;
    match doc {
        Some(d) => {
            let (principals, super_bypass) =
                resolve_auth(&state, &user_id, ctrl.super_permission()).await?;

            if !super_bypass {
                let project_acl = parse_acl(&project_doc).ok();
                if !ctrl.check_hybrid_acl(&d, &principals, Permissions::READ, project_acl.as_ref())
                {
                    return Err(AppError::not_found(format!("{}/{}", kind, id)));
                }
            }

            Ok(Json(ctrl.to_external(d)))
        }
        None => Err(AppError::not_found(format!("{}/{}", kind, id))),
    }
}

/// POST /v1/projects/{project}/{kind}
pub async fn create_scoped_object(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((project_id, kind)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    Json(mut body): Json<Value>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;
    let project_doc = validate_project(&state, &project_id).await?;

    let ctrl = state.controller.for_kind(&kind);
    if !ctrl.is_scoped() {
        return Err(AppError::bad_request(format!(
            "'{}' is not a project-scoped resource kind",
            kind
        )));
    }

    let id = body
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::bad_request("missing 'id' field in request body"))?
        .to_string();

    let (principals, super_bypass) =
        resolve_auth(&state, &user_id, ctrl.super_permission()).await?;

    if !super_bypass {
        // For creation, check project-level CREATE permission (no scope filtering)
        let project_acl = parse_acl(&project_doc).ok();
        let has_create = project_acl.as_ref().map_or(false, |acl| {
            acl.check_permission(&principals, Permissions::CREATE)
        });
        if !has_create {
            return Err(AppError::not_found(format!("{}/{}", kind, id)));
        }
    }

    // Inject project field
    if let Some(obj) = body.as_object_mut() {
        obj.insert(
            "project".to_string(),
            Value::String(project_id.clone()),
        );
    }

    ctrl.prepare_create(&mut body, &user_id);
    state.db.ensure_collection(&kind).await?;

    let mut doc = ctrl.to_internal(body, &state.auth)?;
    let hash = compute_value_hash(&doc);
    if let Some(obj) = doc.as_object_mut() {
        obj.insert("hash_code".to_string(), json!(hash));
    }
    // Use the _key from the doc (may be a composite key set by the controller).
    let final_key = doc
        .get("_key")
        .and_then(|v| v.as_str())
        .unwrap_or(&id)
        .to_string();
    let doc_snapshot = doc.clone();
    state.db.generic_create(&kind, doc).await.map_err(|e| {
        let msg = e.to_string();
        if msg.contains("unique constraint") || msg.contains("1210") {
            AppError::conflict(format!("{}/{} already exists", kind, final_key))
        } else {
            AppError::Internal(e)
        }
    })?;

    ctrl.after_create(&final_key, &user_id, &state.db).await?;

    state.events.entity_lifecycle(EventPriority::Lifecycle, &user_id, &format!("{}/{}", kind, id), "created", None).await;

    Ok((axum::http::StatusCode::CREATED, Json(ctrl.to_external(doc_snapshot))))
}

/// PUT /v1/projects/{project}/{kind}/{id}
pub async fn update_scoped_object(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((project_id, kind, id)): Path<(String, String, String)>,
    State(state): State<Arc<AppState>>,
    Json(mut body): Json<Value>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;
    let project_doc = validate_project(&state, &project_id).await?;

    let ctrl = state.controller.for_kind(&kind);
    if !ctrl.is_scoped() {
        return Err(AppError::bad_request(format!(
            "'{}' is not a project-scoped resource kind",
            kind
        )));
    }

    let db_key = ctrl.scoped_db_key(&project_id, &id);
    let existing = state
        .db
        .generic_get_scoped(&kind, &project_id, &db_key)
        .await?
        .ok_or_else(|| AppError::not_found(format!("{}/{}", kind, id)))?;

    let (principals, super_bypass) =
        resolve_auth(&state, &user_id, ctrl.super_permission()).await?;

    if !super_bypass {
        let project_acl = parse_acl(&project_doc).ok();
        if !ctrl.check_hybrid_acl(
            &existing,
            &principals,
            Permissions::MODIFY,
            project_acl.as_ref(),
        ) {
            return Err(AppError::not_found(format!("{}/{}", kind, id)));
        }
    }

    // Ensure project field and id are set
    if let Some(obj) = body.as_object_mut() {
        obj.insert("id".to_string(), Value::String(id.clone()));
        obj.insert(
            "project".to_string(),
            Value::String(project_id.clone()),
        );
    }

    let mut doc = ctrl.to_internal(body, &state.auth)?;
    let hash = compute_value_hash(&doc);
    if let Some(obj) = doc.as_object_mut() {
        obj.insert("hash_code".to_string(), json!(hash));
    }
    let doc_snapshot = doc.clone();
    state
        .db
        .generic_update(&kind, &db_key, doc)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("document not found") {
                AppError::not_found(format!("{}/{}", kind, id))
            } else {
                AppError::Internal(e)
            }
        })?;

    ctrl.after_update(&db_key, &state.db).await?;

    state.events.entity_lifecycle(EventPriority::Note, &user_id, &format!("{}/{}", kind, id), "updated", None).await;

    Ok(Json(ctrl.to_external(doc_snapshot)))
}

/// POST /v1/projects/{project}/{kind}/{id} — upsert (create or update) a scoped resource.
pub async fn upsert_scoped_object(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((project_id, kind, id)): Path<(String, String, String)>,
    State(state): State<Arc<AppState>>,
    Json(mut body): Json<Value>,
) -> Result<impl IntoResponse, AppError> {
    log::debug!(
        "[HANDLER] upsert_scoped_object: user={}, project={}, kind={}, id={}",
        user_id,
        project_id,
        kind,
        id
    );
    validate_kind(&kind)?;
    let project_doc = validate_project(&state, &project_id).await?;

    let ctrl = state.controller.for_kind(&kind);
    if !ctrl.is_scoped() {
        return Err(AppError::bad_request(format!(
            "'{}' is not a project-scoped resource kind",
            kind
        )));
    }

    // Always set id and project in the body before processing
    if let Some(obj) = body.as_object_mut() {
        obj.insert("id".to_string(), Value::String(id.clone()));
        obj.insert("project".to_string(), Value::String(project_id.clone()));
    }

    let db_key = ctrl.scoped_db_key(&project_id, &id);
    let existing = state
        .db
        .generic_get_scoped(&kind, &project_id, &db_key)
        .await?;
    let is_update = existing.is_some();

    let (principals, super_bypass) =
        resolve_auth(&state, &user_id, ctrl.super_permission()).await?;

    if !super_bypass {
        if is_update {
            let project_acl = parse_acl(&project_doc).ok();
            if !ctrl.check_hybrid_acl(
                existing.as_ref().unwrap(),
                &principals,
                Permissions::MODIFY,
                project_acl.as_ref(),
            ) {
                log::debug!(
                    "[HANDLER] upsert_scoped_object: MODIFY denied for user={}, kind={}, id={}",
                    user_id,
                    kind,
                    id
                );
                return Err(AppError::not_found(format!("{}/{}", kind, id)));
            }
        } else {
            // For creation, check project-level CREATE permission
            let project_acl = parse_acl(&project_doc).ok();
            let has_create = project_acl.as_ref().map_or(false, |acl| {
                acl.check_permission(&principals, Permissions::CREATE)
            });
            if !has_create {
                log::debug!(
                    "[HANDLER] upsert_scoped_object: CREATE denied for user={}, project={}, kind={}, id={}",
                    user_id,
                    project_id,
                    kind,
                    id
                );
                return Err(AppError::not_found(format!("{}/{}", kind, id)));
            }
        }
    }

    if !is_update {
        ctrl.prepare_create(&mut body, &user_id);
    }

    state.db.ensure_collection(&kind).await?;

    let mut doc = ctrl.to_internal(body, &state.auth)?;
    let hash = compute_value_hash(&doc);
    if let Some(obj) = doc.as_object_mut() {
        obj.insert("hash_code".to_string(), json!(hash));
    }

    let doc_snapshot = doc.clone();
    state.db.generic_upsert(&kind, &db_key, doc).await?;

    if is_update {
        if let Err(e) = ctrl.after_update(&db_key, &state.db).await {
            log::error!(
                "[HANDLER] upsert_scoped_object: after_update hook failed: kind={}, id={}, error={}",
                kind, id, e
            );
            return Err(e);
        }
    } else if let Err(e) = ctrl.after_create(&db_key, &user_id, &state.db).await {
        log::error!(
            "[HANDLER] upsert_scoped_object: after_create hook failed: kind={}, id={}, error={}",
            kind, id, e
        );
        return Err(e);
    }

    let action = if is_update { "updated" } else { "created" };
    let priority = if is_update { EventPriority::Note } else { EventPriority::Lifecycle };
    state.events.entity_lifecycle(priority, &user_id, &format!("{}/{}", kind, id), action, None).await;

    let status = if is_update { axum::http::StatusCode::OK } else { axum::http::StatusCode::CREATED };
    Ok((status, Json(ctrl.to_external(doc_snapshot))))
}

/// DELETE /v1/projects/{project}/{kind}/{id}
pub async fn delete_scoped_object(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((project_id, kind, id)): Path<(String, String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    validate_kind(&kind)?;
    let project_doc = validate_project(&state, &project_id).await?;

    let ctrl = state.controller.for_kind(&kind);
    if !ctrl.is_scoped() {
        return Err(AppError::bad_request(format!(
            "'{}' is not a project-scoped resource kind",
            kind
        )));
    }

    let db_key = ctrl.scoped_db_key(&project_id, &id);
    let existing = state
        .db
        .generic_get_scoped(&kind, &project_id, &db_key)
        .await?
        .ok_or_else(|| AppError::not_found(format!("{}/{}", kind, id)))?;

    let (principals, super_bypass) =
        resolve_auth(&state, &user_id, ctrl.super_permission()).await?;

    if !super_bypass {
        let project_acl = parse_acl(&project_doc).ok();
        if !ctrl.check_hybrid_acl(
            &existing,
            &principals,
            Permissions::MODIFY,
            project_acl.as_ref(),
        ) {
            return Err(AppError::not_found(format!("{}/{}", kind, id)));
        }
    }

    state
        .db
        .generic_soft_delete(&kind, &db_key, &user_id)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found or already deleted") {
                AppError::not_found(format!("{}/{}", kind, id))
            } else {
                AppError::Internal(e)
            }
        })?;

    ctrl.after_delete(&db_key, &state.db).await?;

    state.events.entity_lifecycle(EventPriority::Lifecycle, &user_id, &format!("{}/{}", kind, id), "deleted", None).await;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
