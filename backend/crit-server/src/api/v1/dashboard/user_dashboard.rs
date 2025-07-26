use std::sync::Arc;

use axum::{
    extract::State,
    response::IntoResponse,
};

use crate::{errors::AppError, middleware::AuthenticatedUser, models::managers::SpecificUserManager};

pub async fn handle_user_dashboard(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<Arc<crate::state::AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let manager = SpecificUserManager::new(app_state.store.clone(), app_state.index.clone(), &user);
    return Ok(axum::Json(manager.gen_dashboard().await?))
}
