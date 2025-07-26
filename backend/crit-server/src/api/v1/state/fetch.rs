use crate::{
    errors::AppError,
    middleware::AuthenticatedUser,
    models::managers::{ProjectManager, UserManager},
    state::AppState,
    utils::capitalize_first,
};
use axum::{
    body::{Body, to_bytes},
    extract::{Json, Path, Query, State},
    http::{Request, StatusCode},
    response::IntoResponse,
};
use crit_shared::{
    KindOnly,
    entities::{
        Invite, ProjectGitopsSerializable, ProjectGitopsUpdate, UserGitopsSerializable,
        UserGitopsUpdate,
    },
    requests::{IdNs, Ns},
};
use gitops_lib::store::GenericDatabaseProvider;
use std::sync::Arc;

pub async fn handle_list(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<Arc<AppState>>,
    Path(kind): Path<String>,
    Query(namespace): Query<Ns>,
) -> Result<impl IntoResponse, AppError> {
    let kind_cap = capitalize_first(&kind);
    if kind_cap == "User" {
        let manager = UserManager::from_app_state(&app_state);
        return Ok(manager.list_as_response().await?.into_response());
    }
    if kind_cap == "Project" {
        let manager = ProjectManager::from_app_state(&app_state, &user);
        return Ok(manager.list_as_response().await?.into_response());
    }
    if kind_cap == "Invite" {
        if !user.has_admin_status {
            return Err(AppError::AdminCheckFailed);
        }
        let all = app_state
            .store
            .provider::<Invite>()
            .list()
            .await
            .map_err(|e| AppError::from(e))?;
        return Ok(Json(all).into_response());
    }
    return Err(AppError::InvalidData(format!(
        "Unknown kind: '{}'",
        kind_cap
    )));
}

pub async fn handle_describe(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<Arc<AppState>>,
    Path(kind): Path<String>,
    Query(q): Query<IdNs>,
) -> Result<impl IntoResponse, AppError> {
    let kind_cap = capitalize_first(&kind);
    if kind_cap == "User" {
        let manager = UserManager::from_app_state(&app_state);
        return Ok(manager.list_as_response().await?.into_response());
    }
    if kind_cap == "Project" {
        let manager = ProjectManager::from_app_state(&app_state, &user);
        return Ok(Json(manager.describe(&q.id).await?).into_response());
    }
    if kind_cap == "Invite" {
        if !user.has_admin_status {
            return Err(AppError::AdminCheckFailed);
        }
        let all = app_state
            .store
            .provider::<Invite>()
            .get_by_key(&q.id)
            .await
            .map_err(|e| AppError::from(e))?;
        return Ok(Json(all).into_response());
    }
    return Err(AppError::InvalidData(format!(
        "Unknown kind: '{}'",
        kind_cap
    )));
}
