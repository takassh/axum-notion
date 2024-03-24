use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use repositories::RepositoriesError;

pub fn into_response(e: RepositoriesError, message: &str) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("{message}: {e}"))
        .into_response()
}
