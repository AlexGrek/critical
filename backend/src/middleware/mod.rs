use std::sync::Arc;

use axum::{
    body::Body,
    extract::{FromRequestParts, Request, State},
    http::request::Parts,
    middleware::Next,
    response::Response,
};

pub mod auth;

use crate::{error::AppError, middleware::auth::AuthenticatedUser, state::AppState};

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync + 'static, // 'static bound is often needed for extractors in axum 0.8
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user_email = parts
            .extensions
            .get::<String>()
            .cloned()
            .ok_or(AppError::BadRequest(
                "Missing extension: user email".to_string(),
            ))?;

        Ok(AuthenticatedUser(user_email))
    }
}

pub async fn jwt_auth_middleware(
    State(app_state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let (mut __parts__, body) = req.into_parts();
    let path = __parts__.uri.path();

    if path == "/register" || path == "/login" {
        let req = Request::from_parts(__parts__, body);
        return Ok(next.run(req).await);
    }

    // Try to get JWT from Authorization header first
    let token_from_header = __parts__
        .headers
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    // Try to get JWT from cookies if not in header
    let token_from_cookie = __parts__
        .headers
        .get("Cookie")
        .and_then(|h| h.to_str().ok())
        .and_then(|cookies| {
            // Parse cookies and find the JWT token
            cookies
                .split(';')
                .find_map(|cookie| {
                    let cookie = cookie.trim();
                    cookie
                        .strip_prefix("token=")
                        .or_else(|| cookie.strip_prefix("jwt="))
                })
                .map(|s| s.to_string())
        });

    // Use token from header if available, otherwise use token from cookie
    let token = token_from_header
        .or(token_from_cookie)
        .ok_or_else(|| AppError::Authorization("Unauthorized".to_string()))?;

    match app_state.auth.decode_token(&token) {
        Ok(claims) => {
            if app_state.controller.user.validate_user(&claims.sub).await {
                __parts__.extensions.insert(claims.sub);
                let req = Request::from_parts(__parts__, body);
                Ok(next.run(req).await)
            } else {
                log::warn!("User invalid: {}", &claims.sub);
                Err(AppError::Authorization("Unauthorized".to_string()))
            }
        }
        Err(e) => {
            log::warn!("JWT validation failed: {}", e);
            Err(AppError::Authorization("Unauthorized".to_string()))
        }
    }
}

pub async fn token_auth_middleware_mgmt(
    State(app_state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let (parts, body) = req.into_parts();

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

    let token = token.ok_or(AppError::Authorization("Unauthorized".to_string()))?;

    if token == app_state.config.management_token {
        let req = Request::from_parts(parts, body);
        Ok(next.run(req).await)
    } else {
        Err(AppError::Authorization("Unauthorized".to_string()))
    }
}

pub async fn apikey_auth_middleware_user(
    State(app_state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let headers = req.headers();

    // Try to get API key from X-Api-Key header first, then Authorization header
    let api_key = headers
        .get("X-Api-Key")
        .or_else(|| headers.get("Authorization"))
        .and_then(|h| h.to_str().ok())
        .map(|s| {
            // Remove "Bearer " prefix if present in Authorization header
            s.strip_prefix("Bearer ").unwrap_or(s).to_string()
        })
        .ok_or_else(|| AppError::Authorization("Missing API key in headers".to_string()))?;

    if !app_state.config.client_api_keys.contains(&api_key) {
        return Err(AppError::Authorization("Unauthorized: Invalid API key".to_string()));
    }

    Ok(next.run(req).await)
}
