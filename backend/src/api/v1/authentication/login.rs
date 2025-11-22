use crate::{
    error::AppError,
    schema::{Created, LoginRequest, LoginResponse, RegisterRequest, User},
    state::AppState,
    validation::naming::validate_username,
};
use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/register",
    responses((status = 201, description = "Create user successfully"))
)]
pub async fn register(
    State(app_state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Created, AppError>{
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

    app_state.db.users().create_user(user.into()).await?;

    log::info!(
        "Register event -> {}",
        format!("User with ID {:?} created: {}", &uid, &req.user)
    );

    Ok(Created{})
}

pub async fn login(
    State(app_state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = app_state
        .db
        .users()
        .get_user(&req.user)
        .await
        .map_err(|_e| AppError::Authorization("Unauthorized".to_string()))?;

    if !app_state
        .auth
        .verify_password(&req.password, &user.password_hash)?
    {
        return Err(AppError::Authorization("Unauthorized".to_string()));
    }

    let token = app_state.auth.create_token(&user.username)?;

    log::info!(
        "Auth event -> {}",
        format!("User logged in: {}", &user.username)
    );

    Ok(Json(LoginResponse { token: token.0 }))
}
