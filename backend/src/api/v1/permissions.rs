use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use serde_json::{Value, json};

use crit_shared::util_models::super_permissions;

use crate::{error::AppError, middleware::auth::AuthenticatedUser, state::AppState};

#[derive(Deserialize)]
pub struct PermissionBody {
    /// One or more principal IDs (user or group keys) to grant/revoke the permission.
    /// Accepts both `"principals": [...]` (preferred) and `"principal": "..."` (convenience).
    #[serde(default)]
    principals: Vec<String>,
    #[serde(default)]
    principal: Option<String>,
}

impl PermissionBody {
    fn resolved_principals(self) -> Vec<String> {
        let mut all = self.principals;
        if let Some(p) = self.principal {
            if !all.contains(&p) {
                all.push(p);
            }
        }
        all
    }
}

/// Check that the requesting user is authorized to manage permissions.
/// Requires either ADM_USER_MANAGER or ADM_GODMODE.
async fn require_permission_manager(
    state: &AppState,
    user_id: &str,
) -> Result<(), AppError> {
    let principals = state.get_cached_principals(user_id).await.map_err(AppError::Internal)?;

    let has_user_manager = state
        .db
        .has_permission_with_principals(&principals, super_permissions::ADM_USER_MANAGER)
        .await
        .map_err(AppError::Internal)?;

    let has_godmode = state
        .db
        .has_permission_with_principals(&principals, super_permissions::ADM_GODMODE)
        .await
        .map_err(AppError::Internal)?;

    if !has_user_manager && !has_godmode {
        log::debug!(
            "[PERMISSIONS] grant/revoke denied for {}: lacks ADM_USER_MANAGER and ADM_GODMODE",
            user_id
        );
        return Err(AppError::forbidden("requires ADM_USER_MANAGER or ADM_GODMODE"));
    }

    log::debug!("[PERMISSIONS] grant/revoke authorized for {}", user_id);
    Ok(())
}

/// Grant a named permission to one or more principals.
///
/// `POST /v1/global/permissions/{key}/grant`
/// Body: `{"principals": ["u_alice", "g_admins"]}` or `{"principal": "u_alice"}`
///
/// Requires ADM_USER_MANAGER or ADM_GODMODE.
/// Silently succeeds if the principal(s) already hold the permission.
pub async fn grant_permission(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(key): Path<String>,
    Json(body): Json<PermissionBody>,
) -> Result<Json<Value>, AppError> {
    require_permission_manager(&state, &user_id).await?;

    let principals = body.resolved_principals();
    if principals.is_empty() {
        return Err(AppError::BadRequest(
            "at least one principal must be provided".to_string(),
        ));
    }

    // Grant the permission to each principal.
    for principal in &principals {
        state
            .db
            .grant_permission(&key, principal)
            .await
            .map_err(AppError::Internal)?;
    }

    log::info!(
        "[PERMISSIONS] {} granted '{}' to {:?}",
        user_id,
        key,
        principals
    );

    Ok(Json(json!({
        "permission": key,
        "granted_to": principals,
    })))
}

/// Revoke a named permission from one or more principals.
///
/// `POST /v1/global/permissions/{key}/revoke`
/// Body: `{"principals": ["u_alice"]}` or `{"principal": "u_alice"}`
///
/// Requires ADM_USER_MANAGER or ADM_GODMODE.
/// Silently succeeds if the principal(s) don't hold the permission.
pub async fn revoke_permission(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(key): Path<String>,
    Json(body): Json<PermissionBody>,
) -> Result<Json<Value>, AppError> {
    require_permission_manager(&state, &user_id).await?;

    let principals = body.resolved_principals();
    if principals.is_empty() {
        return Err(AppError::BadRequest(
            "at least one principal must be provided".to_string(),
        ));
    }

    for principal in &principals {
        state
            .db
            .revoke_permission(&key, principal)
            .await
            .map_err(AppError::Internal)?;
    }

    log::info!(
        "[PERMISSIONS] {} revoked '{}' from {:?}",
        user_id,
        key,
        principals
    );

    Ok(Json(json!({
        "permission": key,
        "revoked_from": principals,
    })))
}
