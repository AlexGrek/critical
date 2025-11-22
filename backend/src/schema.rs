use axum::{http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct User {
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub user: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub user: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImpersonateRequest {
    pub action: String,
    pub user: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(ToSchema)]
pub struct Created;

impl IntoResponse for Created {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        StatusCode::CREATED.into_response()
    }
}

impl utoipa::IntoResponses for Created {
    fn responses() -> std::collections::BTreeMap<String, utoipa::openapi::RefOr<utoipa::openapi::Response>> {
        use utoipa::openapi::{ResponseBuilder, RefOr};
        let mut responses = std::collections::BTreeMap::new();
        responses.insert(
            "201".to_string(),
            RefOr::T(
                ResponseBuilder::new()
                    .description("Created")
                    .build(),
            ),
        );
        responses
    }
}