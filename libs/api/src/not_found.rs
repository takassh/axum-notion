use axum::http::StatusCode;

pub(super) async fn get_404() -> StatusCode {
    StatusCode::NOT_FOUND
}
