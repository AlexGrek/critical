use std::collections::BTreeMap;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::json;
use thiserror::Error;
use utoipa::{
    IntoResponses, PartialSchema, ToSchema,
    openapi::{self, ContentBuilder, RefOr, ResponseBuilder},
};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Authorization failed: {0}")]
    Authorization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Scheduling impossible: {0}")]
    SchedulingImpossible(String),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Bcrypt error: {0}")]
    BcryptError(#[from] bcrypt::BcryptError),
}

impl AppError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Serialization(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Authentication(_) => StatusCode::UNAUTHORIZED,
            AppError::Authorization(_) => StatusCode::UNAUTHORIZED,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Jwt(_) => StatusCode::UNAUTHORIZED,
            AppError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Parse(_) => StatusCode::BAD_REQUEST,
            AppError::BcryptError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::SchedulingImpossible(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    /// Get error type as string for JSON responses
    pub fn error_type(&self) -> &'static str {
        match self {
            AppError::Internal(_) => "internal_error",
            AppError::Serialization(_) => "serialization_error",
            AppError::Authentication(_) => "authentication_error",
            AppError::Authorization(_) => "authorization_error",
            AppError::Validation(_) => "validation_error",
            AppError::NotFound(_) => "not_found",
            AppError::Conflict(_) => "conflict",
            AppError::BadRequest(_) => "bad_request",
            AppError::Jwt(_) => "jwt_error",
            AppError::Io(_) => "io_error",
            AppError::Parse(_) => "parse_error",
            AppError::BcryptError(_) => "bcrypt_error",
            AppError::SchedulingImpossible(_) => "scheduling impossible",
        }
    }

    /// Check if this error should be logged
    pub fn should_log(&self) -> bool {
        match self {
            AppError::Authentication(_)
            | AppError::Authorization(_)
            | AppError::NotFound(_)
            | AppError::BadRequest(_)
            | AppError::Jwt(_)
            | AppError::Parse(_) => false,
            AppError::Validation(_)
            | AppError::Internal(_)
            | AppError::Serialization(_)
            | AppError::Io(_)
            | AppError::Conflict(_)
            | AppError::BcryptError(_) => true,
            AppError::SchedulingImpossible(_) => true,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub r#type: String, // Use r# to allow "type" keyword
    pub message: String,
    pub status: u16,
}

impl IntoResponses for AppError {
    fn responses() -> BTreeMap<String, RefOr<openapi::Response>> {
        let mut responses = BTreeMap::new();

        // 400 Bad Request
        responses.insert(
            "400".to_string(),
            RefOr::T(
                ResponseBuilder::new()
                    .description("Bad Request")
                    .content(
                        "application/json",
                        ContentBuilder::new()
                            .schema(Some(ErrorResponse::schema()))
                            .build(),
                    )
                    .build(),
            ),
        );

        // 401 Unauthorized
        responses.insert(
            "401".to_string(),
            RefOr::T(
                ResponseBuilder::new()
                    .description("Unauthorized")
                    .content(
                        "application/json",
                        ContentBuilder::new()
                            .schema(Some(ErrorResponse::schema()))
                            .build(),
                    )
                    .build(),
            ),
        );

        // 404 Not Found
        responses.insert(
            "404".to_string(),
            RefOr::T(
                ResponseBuilder::new()
                    .description("Not Found")
                    .content(
                        "application/json",
                        ContentBuilder::new()
                            .schema(Some(ErrorResponse::schema()))
                            .build(),
                    )
                    .build(),
            ),
        );

        // 409 Conflict
        responses.insert(
            "409".to_string(),
            RefOr::T(
                ResponseBuilder::new()
                    .description("Conflict")
                    .content(
                        "application/json",
                        ContentBuilder::new()
                            .schema(Some(ErrorResponse::schema()))
                            .build(),
                    )
                    .build(),
            ),
        );

        // 500 Internal Server Error
        responses.insert(
            "500".to_string(),
            RefOr::T(
                ResponseBuilder::new()
                    .description("Internal Server Error")
                    .content(
                        "application/json",
                        ContentBuilder::new()
                            .schema(Some(ErrorResponse::schema()))
                            .build(),
                    )
                    .build(),
            ),
        );

        // 503 Service Unavailable
        responses.insert(
            "503".to_string(),
            RefOr::T(
                ResponseBuilder::new()
                    .description("Service Unavailable")
                    .content(
                        "application/json",
                        ContentBuilder::new()
                            .schema(Some(ErrorResponse::schema()))
                            .build(),
                    )
                    .build(),
            ),
        );

        responses
    }
}

// Implement IntoResponse for Axum
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        // Log server errors
        if self.should_log() {
            tracing::error!("AppError: {} (status: {})", self, status);
        } else {
            tracing::debug!("AppError: {} (status: {})", self, status);
        }

        let body = json!({
            "error": {
                "type": self.error_type(),
                "message": self.to_string(),
                "status": status.as_u16()
            }
        });

        (status, Json(body)).into_response()
    }
}

// Convenience constructors
impl AppError {
    pub fn authentication<T: std::fmt::Display>(msg: T) -> Self {
        Self::Authentication(msg.to_string())
    }

    pub fn authorization<T: std::fmt::Display>(msg: T) -> Self {
        Self::Authorization(msg.to_string())
    }

    pub fn validation<T: std::fmt::Display>(msg: T) -> Self {
        Self::Validation(msg.to_string())
    }

    pub fn not_found<T: std::fmt::Display>(msg: T) -> Self {
        Self::NotFound(msg.to_string())
    }

    pub fn conflict<T: std::fmt::Display>(msg: T) -> Self {
        Self::Conflict(msg.to_string())
    }

    pub fn bad_request<T: std::fmt::Display>(msg: T) -> Self {
        Self::BadRequest(msg.to_string())
    }

    pub fn serialization<T: std::fmt::Display>(msg: T) -> Self {
        Self::Serialization(msg.to_string())
    }

    pub fn parse<T: std::fmt::Display>(msg: T) -> Self {
        Self::Parse(msg.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for AppError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::Parse(format!("UTF-8 conversion error: {}", err))
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::Parse(format!("Integer parse error: {}", err))
    }
}

// Type alias for Results
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            AppError::authentication("test").status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AppError::authorization("test").status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AppError::validation("test").status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::not_found("test").status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AppError::conflict("test").status_code(),
            StatusCode::CONFLICT
        );
    }
}
