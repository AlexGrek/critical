use axum::{extract::{State, Json}, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use gitops_lib::store::GenericDatabaseProvider;
use std::{collections::HashMap, sync::Arc};
use crate::{
    auth::invites::use_registration_invite, errors::AppError, middleware::AuthenticatedUser, models::{entities::User, LoginRequest, LoginResponse, RegisterRequest}, state::AppState
};

pub async fn register(
    State(app_state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    use_registration_invite(&app_state, &req.invite_id, &req.invite_key).await?;

    let hashed_password = app_state.auth.hash_password(&req.password)?;

    let user = User {
        uid: req.uid.clone(),
        password_hash: Some(hashed_password),
        annotations: HashMap::new(),
        has_admin_status: false,
        email: req.email.clone(),
        oauth: None,
        created_at: Utc::now().to_rfc3339(),
    };

    app_state.store.provider::<User>().insert(&user).await?;

    log::info!("Auth event -> {}", format!("User with ID {:?} created: {}", &req.uid, &req.email));

    Ok(StatusCode::OK)
}

pub async fn login(
    State(app_state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_provider = app_state.store.provider::<User>();
    let user = user_provider.try_get_by_key(&req.uid).await?
        .ok_or(AppError::InvalidCredentials)?;

    if !app_state.auth.verify_password(&req.password, &user.password_hash.unwrap_or("".to_string()))? {
        return Err(AppError::InvalidCredentials);
    }

    let token = app_state.auth.create_token(&user.uid)?;

    log::info!("Auth event -> {}", format!("User logged in: {}", &user.uid));

    Ok(Json(LoginResponse { token }))
}

pub async fn get_protected_data(
    AuthenticatedUser(_user_email): AuthenticatedUser,
    State(_app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json("Dummy protected data"))
}