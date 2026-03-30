use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub error: ApiErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorDetail {
    pub code: String,
    pub message: String,
}

#[derive(Debug)]
pub enum AppError {
    NotConnected,
    TokenExpired,
    QboRequest(String),
    Internal(String),
    NotFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::NotConnected => (
                StatusCode::UNAUTHORIZED,
                "NOT_CONNECTED",
                "QuickBooks is not connected. Please connect first.".into(),
            ),
            AppError::TokenExpired => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "QuickBooks connection expired. Please reconnect.".into(),
            ),
            AppError::QboRequest(msg) => (StatusCode::BAD_GATEWAY, "QBO_ERROR", msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
        };

        let body = ApiErrorBody {
            error: ApiErrorDetail {
                code: code.into(),
                message,
            },
        };

        (status, Json(body)).into_response()
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::QboRequest(err.to_string())
    }
}
