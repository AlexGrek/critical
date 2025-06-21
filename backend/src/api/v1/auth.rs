use axum::{extract::{State, Json}, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use std::{collections::HashMap, sync::Arc};
use crate::{
    errors::AppError, middleware::AuthenticatedUser, models::{LoginRequest, LoginResponse, RegisterRequest}, state::AppState
};
use uuid::Uuid;

pub async fn register(
    State(app_state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    let hashed_password = app_state.auth.hash_password(&req.password)?;

    let id = Uuid::new_v4();

    // let user = User {
    //     email: req.email.clone(),
    //     password_hash: Some(hashed_password),
    //     metadata: HashMap::new(),
    //     admin: None, // TODO: use admins txt
    //     oauth_id: None,
    //     created_at: Utc::now()
    // };

    // app_state.db.upsert_user(user)?;

    log::info!("Auth event -> {}", format!("User with ID {:?} created: {}", id, &req.email));

    Ok(StatusCode::OK)
}

pub async fn login(
    State(app_state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    // let user = app_state.db.get_user(&req.email)?
    //     .ok_or(AppError::InvalidCredentials)?;

    // if !app_state.auth.verify_password(&req.password, &user.password_hash.unwrap_or("".to_string()))? {
    //     return Err(AppError::InvalidCredentials);
    // }

    // let token = app_state.auth.create_token(&user.email)?;

    // log::info!("Auth event -> {}", format!("User logged in: {}", &user.email));

    // Ok(Json(LoginResponse { token }))

    Ok(())
}

pub async fn get_protected_data(
    AuthenticatedUser(_user_email): AuthenticatedUser,
    State(_app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json("Dummy protected data"))
}