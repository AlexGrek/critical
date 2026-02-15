use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use chrono::Utc;

use crate::{
    error::AppError,
    models,
    schema::{Created, LoginRequest, LoginResponse, RegisterRequest, User},
    state::AppState,
    validation::naming::validate_username,
};

impl From<User> for models::User {
    fn from(src: User) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("registered_at".to_string(), Utc::now().to_rfc3339());

        Self {
            id: format!("u_{}", src.username),
            password_hash: src.password_hash,
            metadata,
            ..Self::default()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/register",
    responses((status = 201, description = "Create user successfully"))
)]
pub async fn register(
    State(app_state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Created, AppError> {
    if !app_state.runtime_config.user_login_allowed {
        return Err(AppError::Authentication(
            "Only admin can create new users".to_string(),
        ));
    }

    let hashed_password = app_state.auth.hash_password(&req.password)?;

    let user = User {
        username: validate_username(&req.user).map_err(AppError::Validation)?,
        password_hash: hashed_password,
    };

    let uid = user.username.clone();

    app_state.db.create_user(user.into(), None).await?;

    log::info!(
        "Register event -> {}",
        format!("User with ID {:?} created: {}", &uid, &req.user)
    );

    Ok(Created {})
}

pub async fn login(
    State(app_state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = app_state
        .db
        .get_user_by_id(&req.user)
        .await
        .map_err(|_e| AppError::Authorization("Unauthorized".to_string()))?;

    let true_user = user.ok_or(AppError::Authorization("Unauthorized".to_string()))?;

    if !app_state
        .auth
        .verify_password(&req.password, &true_user.password_hash)?
    {
        return Err(AppError::Authorization("Unauthorized".to_string()));
    }

    let token = app_state.auth.create_token(&true_user.id)?;

    log::info!(
        "Auth event -> {}",
        format!("User logged in: {}", &true_user.id)
    );

    Ok(Json(LoginResponse { token: token.0 }))
}
