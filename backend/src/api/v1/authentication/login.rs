use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::{HeaderMap, HeaderValue, header},
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

    let user_model: models::User = user.into();
    let user_id = user_model.id.clone();
    app_state.db.create_user(user_model, None).await?;

    // Grant default permissions to new users
    app_state
        .db
        .grant_permission(models::super_permissions::USR_CREATE_PROJECTS, &user_id)
        .await
        .map_err(|e| AppError::Internal(e))?;

    app_state
        .db
        .grant_permission(models::super_permissions::USR_CREATE_GROUPS, &user_id)
        .await
        .map_err(|e| AppError::Internal(e))?;

    log::info!("Register event -> User with ID {:?} created: {}", &uid, &req.user);

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

    let (token_str, exp) = app_state.auth.create_token(&true_user.id)?;

    log::info!("Auth event -> User logged in: {}", &true_user.id);

    // Calculate max-age from expiration timestamp
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    let max_age = exp.saturating_sub(now);

    let cookie = format!(
        "token={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age={}",
        token_str, max_age
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|_| {
            AppError::Internal(anyhow::anyhow!("Failed to build Set-Cookie header"))
        })?,
    );

    Ok((headers, Json(LoginResponse { token: token_str })))
}

pub async fn logout() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    // Expire the token cookie immediately
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_static(
            "token=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0",
        ),
    );
    (headers, axum::http::StatusCode::NO_CONTENT)
}
