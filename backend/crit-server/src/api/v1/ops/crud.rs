use crate::{
    errors::AppError,
    middleware::AuthenticatedUser,
    models::managers::{ProjectManager, UserManager},
    state::AppState,
};
use axum::{
    body::{Body, to_bytes},
    extract::{Json, Path, Query, State},
    http::{Request, StatusCode},
    response::IntoResponse,
};
use crit_shared::{
    KindOnly,
    entities::{ProjectGitopsUpdate, UserGitopsUpdate},
    requests::Ns,
};
use gitops_lib::store::GenericDatabaseProvider;
use std::sync::Arc;

pub async fn handle_create(
    AuthenticatedUser(_user): AuthenticatedUser,
    State(app_state): State<Arc<AppState>>,
    req: Request<Body>,
) -> Result<impl IntoResponse, AppError> {
    let body = to_bytes(req.into_body(), 10000000).await;

    let Ok(bytes) = body else {
        return Err(AppError::BadRequest("No body in request".to_string()));
    };

    // Parse just the "kind" field
    let kind_result: Result<KindOnly, _> = serde_json::from_slice(&bytes);
    let kind = match kind_result {
        Ok(k) => k.kind,
        Err(_) => {
            return Err(AppError::BadRequest("Kind was not parsed".to_string()));
        }
    };

    // Dispatch based on kind
    let result: Result<(), AppError> = match kind.as_str() {
        "user" => serde_json::from_slice::<UserGitopsUpdate>(&bytes)
            .map(|item| UserManager::new(app_state.store.clone()).create(item))
            .map_err(|e| e.into()),
        "project" => serde_json::from_slice::<ProjectGitopsUpdate>(&bytes)
            .map(|_| ())
            .map_err(|e| e.into()),
        kind => Err(AppError::InvalidData(format!("Unknown kind: '{}'", kind))),
    };

    result
}

pub async fn handle_list(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<Arc<AppState>>,
    Path(kind): Path<String>,
    Query(namespace): Query<Ns>,
) -> Result<impl IntoResponse, AppError> {
    if kind == "user" {
        let manager = UserManager::new(app_state.store.clone());
        return Ok(manager.list_as_response().await?.into_response());
    }
    if kind == "project" {
        let manager = ProjectManager::new(app_state.store.clone(), &user);
        return Ok(manager.list_as_response().await?.into_response());
    }
    return Err(AppError::InvalidData(format!("Unknown kind: '{}'", kind)));
}
