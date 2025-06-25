// src/middleware/mod.rs
use axum::{
    body::Body, // Explicitly use axum's Body type
    extract::{FromRequestParts, State},
    http::{request::Parts, Request},
    middleware::Next, // Import Next without generic
    response::Response,
};
use gitops_lib::store::GenericDatabaseProvider;
// Removed: use tower_http::handle_error::HandleErrorLayer; // Not used in this file

use crate::{errors::AppError, models::entities::User, state::AppState};

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
// Removed: use async_trait::async_trait; // Not needed for native async traits

// Custom extractor to get the authenticated user email from extensions
pub struct AuthenticatedUserEmail(pub String);

pub struct AuthenticatedUser(pub User);

// Removed #[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync + 'static, // 'static bound is often needed for extractors in axum 0.8
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user = parts
            .extensions
            .get::<User>()
            .cloned()
            .ok_or(AppError::MissingExtension("user type".to_string()))?;
        Ok(AuthenticatedUser(user))
    }
}

impl<S> FromRequestParts<S> for AuthenticatedUserEmail
where
    S: Send + Sync + 'static, // 'static bound is often needed for extractors in axum 0.8
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user_email = parts
            .extensions
            .get::<String>()
            .cloned()
            .ok_or(AppError::MissingExtension("user email".to_string()))?;

        Ok(AuthenticatedUserEmail(user_email))
    }
}

pub async fn jwt_auth_middleware(
    State(app_state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let (mut parts, body) = req.into_parts();

    let path = parts.uri.path();

    if path == "/register" || path == "/login" {
        let req = Request::from_parts(parts, body);
        return Ok(next.run(req).await);
    }

    let auth_header = parts
        .headers
        .get("Authorization")
        .and_then(|header| header.to_str().ok());

    let token =
        auth_header.and_then(|header| header.strip_prefix("Bearer ").map(|s| s.to_string()));

    let token = token.ok_or(AppError::Unauthorized)?;

    match app_state.auth.decode_token(&token) {
        Ok(claims) => {
            parts.extensions.insert(claims.sub.clone());
            // insert actual user
            let user = app_state
                .store.provider::<User>()
                .try_get_by_key(&claims.sub).await?;

            match user {
                Some(u) => {
                    parts.extensions.insert(u);
                    ()
                },
                _ => return Err(AppError::UserNotFound)
            }

            let req = Request::from_parts(parts, body);
            Ok(next.run(req).await)
        }
        Err(e) => {
            log::warn!("JWT validation failed: {}", e);
            Err(AppError::Unauthorized)
        }
    }
}

// Middleware to check if the authenticated user is an admin
// Signature: (AuthenticatedUser, State<Arc<AppState>>, Request<Body>, Next)
pub async fn admin_check_middleware(
    AuthenticatedUser(user): AuthenticatedUser, // Extractor 1
    State(app_state): State<Arc<AppState>>,           // Extractor 2
    req: Request<Body>,                               // Request<Body>
    next: Next,                                       // Next
) -> Result<Response, AppError> {
    // Read admin list EVERY time as requested
    let admins_file = File::open(&app_state.admin_file_path)
        .map_err(|_| AppError::ConfigError("Could not open admins.txt".into()))?;
    let reader = BufReader::new(admins_file);
    let admins: HashSet<String> = reader.lines().filter_map(|line| line.ok()).collect();

    if admins.contains(&user.email) {
        Ok(next.run(req).await)
    } else {
        Err(AppError::AdminCheckFailed)
    }
}
