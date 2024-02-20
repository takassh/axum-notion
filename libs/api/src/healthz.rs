use axum::http::StatusCode;

pub(super) async fn get_health() -> StatusCode {
    StatusCode::OK
}
