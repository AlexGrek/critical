use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use gitops_lib::GitopsResourceRoot;

use crate::{auth::invites::create_invite, errors::AppError, middleware::AuthenticatedUser, state::AppState};

pub async fn issue_invite(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let invite = create_invite(&app_state, &user).await?;
    Ok(Json(invite.into_serializable()))
}
