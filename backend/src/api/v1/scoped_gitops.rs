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
use crit_shared::util_models::Permissions;

use super::gitops::{ListQuery, validate_kind};

/// Validate that a project exists and is not deleted. Returns the project doc.
async fn validate_project(state: &AppState, project_id: &str) -> Result<Value, AppError> {
    let project = state.db.generic_get("projects", project_id).await?;
    project.ok_or_else(|| AppError::not_found(format!("projects/{}", project_id)))
}

/// Resolve user principals and check super-permission bypass for a controller.
async fn resolve_auth(
    state: &AppState,
    user_id: &str,
    super_perm: Option<&str>,
) -> Result<(Vec<String>, bool), AppError> {
    let principals = state.db.get_user_principals(user_id).await?;
    let super_bypass = match super_perm {
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
            ctrl.resource_kind_name(),
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

    let doc = state
        .db
        .generic_get_scoped(&kind, &project_id, &id)
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
        // For creation, check project-level CREATE permission
        let project_acl = parse_acl(&project_doc).ok();
        let has_create = project_acl.as_ref().map_or(false, |acl| {
            acl.check_permission_scoped(&principals, Permissions::CREATE, ctrl.resource_kind_name())
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

    let doc = ctrl.to_internal(body, &state.auth)?;
    state.db.generic_create(&kind, doc).await.map_err(|e| {
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

    let existing = state
        .db
        .generic_get_scoped(&kind, &project_id, &id)
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

    let existing = state
        .db
        .generic_get_scoped(&kind, &project_id, &id)
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
