
use axum::{
    extract::{Request},
    http::{self},
    middleware::{Next},
    response::{Response},
};


use crate::{ApiError, ACCEPT_API_KEY};

pub async fn auth(req: Request, next: Next) -> Result<Response, ApiError> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(ApiError::AuthError(
            "Authorization header is missing".to_string(),
        ));
    };

    let accept_api_key = ACCEPT_API_KEY.get().unwrap();

    if accept_api_key == auth_header {
        return Ok(next.run(req).await);
    }

    Err(ApiError::AuthError("Invalid API key".to_string()))
}
