use std::fs;

use axum::{http::StatusCode, response::IntoResponse};
use serde_json::Value;
use tracing::error;
use util::workspace_dir;

use crate::ApiError;

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status_code;
        let mut _message = "".to_string();

        match self {
            ApiError::AuthError(message) => {
                status_code = StatusCode::UNAUTHORIZED;
                _message = message;
            }
            ApiError::ClientError(message) => {
                status_code = StatusCode::BAD_REQUEST;
                _message = message;
            }
            ApiError::ServerError(message) => {
                status_code = StatusCode::INTERNAL_SERVER_ERROR;
                _message = message;
            }
        }
        (status_code, _message).into_response()
    }
}

pub type ApiResponse<T> = Result<T, ApiError>;

pub trait IntoApiResponse<T> {
    fn into_response(self, error_code: &str) -> ApiResponse<T>;
}

impl<T> IntoApiResponse<T> for anyhow::Result<T> {
    fn into_response(self, error_code: &str) -> ApiResponse<T> {
        self.map_err(|e| {
            error!("{:?}", e);
            let errors = fs::read_to_string(
                workspace_dir().join("libs/api/src/error-code.json"),
            )
            .unwrap();
            let parsed: Value = serde_json::from_str(&errors).unwrap();
            let errors = parsed.as_object().unwrap().clone();

            let first_char = error_code.as_bytes().first();

            match first_char {
                Some(&b'4') => {
                    return ApiError::ClientError(
                        errors[error_code].as_str().unwrap().to_string(),
                    );
                }
                _ => {
                    return ApiError::ServerError(
                        errors[error_code].as_str().unwrap().to_string(),
                    );
                }
            }
        })
    }
}
