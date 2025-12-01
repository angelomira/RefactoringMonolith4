/// Unified error handling module
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;

/// Unified error response format
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub ok: bool,
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ApiError {
    Database(sqlx::Error),
    ExternalApi(reqwest::Error),
    NotFound(String),
    Internal(String),
    InvalidInput(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Database(e) => write!(f, "Database error: {}", e),
            ApiError::ExternalApi(e) => write!(f, "External API error: {}", e),
            ApiError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ApiError::Internal(msg) => write!(f, "Internal error: {}", msg),
            ApiError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        ApiError::Database(err)
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::ExternalApi(err)
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::Internal(err.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (code, message) = match &self {
            ApiError::Database(e) => ("DATABASE_ERROR", e.to_string()),
            ApiError::ExternalApi(e) => {
                if let Some(status) = e.status() {
                    (
                        match status.as_u16() {
                            403 => "UPSTREAM_403",
                            404 => "UPSTREAM_404",
                            429 => "UPSTREAM_429",
                            500..=599 => "UPSTREAM_5XX",
                            _ => "UPSTREAM_ERROR",
                        },
                        format!("External API error: {}", e),
                    )
                } else {
                    ("UPSTREAM_ERROR", format!("External API error: {}", e))
                }
            }
            ApiError::NotFound(msg) => ("NOT_FOUND", msg.clone()),
            ApiError::Internal(msg) => ("INTERNAL_ERROR", msg.clone()),
            ApiError::InvalidInput(msg) => ("INVALID_INPUT", msg.clone()),
        };

        let error_response = ErrorResponse {
            ok: false,
            error: ErrorDetail {
                code: code.to_string(),
                message,
                trace_id: None, // TODO: implement trace ID generation
            },
        };

        // Always return HTTP 200 with ok=false as per requirements
        (StatusCode::OK, Json(error_response)).into_response()
    }
}

/// Type alias for API results
pub type ApiResult<T> = Result<T, ApiError>;
