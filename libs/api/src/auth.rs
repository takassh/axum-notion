use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Context;
use axum::{
    extract::{Request, State},
    http,
    middleware::Next,
    response::Response,
};
use entity::user::User;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

use crate::{
    response::IntoApiResponse, ApiError, ApiState, ADMIN_USER, JWKS_URL,
};

use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet},
    DecodingKey, Validation,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub user_id: Option<i32>,
}

static JWKS: OnceCell<JwkSet> = OnceCell::const_new();

pub async fn set_user_id(
    State(state): State<Arc<ApiState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let mut claims = req.extensions().get::<Claims>().unwrap().clone();
    let user = state
        .repo
        .user
        .find_by_sub(&claims.sub)
        .await
        .into_response("501-013")?;

    let Some(user) = user else {
        let id = state
            .repo
            .user
            .save(User {
                sub: claims.sub.clone(),
                ..Default::default()
            })
            .await
            .into_response("502-013")?;

        let user = state.repo.user.find_by_id(id).await;

        let Ok(Some(user)) = user else {
            return Err(anyhow!("failed to get user. id: {}", id))
                .into_response("502-013");
        };

        claims.user_id = Some(user.id);
        req.extensions_mut().insert(claims);
        return Ok(next.run(req).await);
    };

    claims.user_id = Some(user.id);
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

pub async fn rate_limit(
    State(state): State<Arc<ApiState>>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let claims = req.extensions().get::<Claims>().unwrap();
    if ADMIN_USER.get().unwrap() == claims.sub.as_str() {
        return Ok(next.run(req).await);
    }
    let count = state.repo.session.as_ref().unwrap().increment(&claims.sub);
    let Ok(count) = count else {
        return Err(ApiError::ServerError("internal error".to_string()));
    };
    if count > 10 {
        return Err(ApiError::AuthError("reached rate limit".to_string()));
    }

    Ok(next.run(req).await)
}

pub async fn user_auth(
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let Ok(token) = get_authorization_header(&req) else {
        return Err(ApiError::AuthError("failed to authorization".to_string()));
    };

    let Ok(claims) = validate_token(&token).await else {
        return Err(ApiError::AuthError("invalid token".to_string()));
    };

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

pub async fn admin_auth(
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let Ok(token) = get_authorization_header(&req) else {
        return Err(ApiError::AuthError("failed to authorization".to_string()));
    };

    let Ok(claims) = validate_token(&token).await else {
        return Err(ApiError::AuthError("invalid token".to_string()));
    };
    if ADMIN_USER.get().unwrap() == claims.sub.as_str() {
        req.extensions_mut().insert(claims);
        return Ok(next.run(req).await);
    }

    Err(ApiError::AuthError("you don't have access".to_string()))
}

async fn get_jwks() -> anyhow::Result<JwkSet> {
    let jwks_url = JWKS_URL.get().context("failed to authorization")?;
    let jwks: JwkSet =
        serde_json::from_str(&reqwest::get(jwks_url).await?.text().await?)?;
    Ok(jwks)
}

fn get_authorization_header(req: &Request) -> anyhow::Result<String> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .context("failed to get AUTHORIZATION")?
        .to_str()?;

    Ok(auth_header
        .split_whitespace()
        .nth(1)
        .context("failed to get token")?
        .to_string())
}

async fn validate_token(token: &str) -> anyhow::Result<Claims> {
    let jwks = match JWKS.get() {
        Some(jwks) => jwks.clone(),
        None => {
            let jwks = get_jwks().await?;
            JWKS.set(jwks.clone()).unwrap();
            jwks
        }
    };

    let header = decode_header(token)?;

    let kid = header.kid.context("failed to find kid")?;

    let jwk = jwks.find(&kid).context("failed to find jwk")?;

    let decoding_key = match &jwk.algorithm {
        AlgorithmParameters::RSA(rsa) => {
            DecodingKey::from_rsa_components(&rsa.n, &rsa.e).unwrap()
        }
        _ => unreachable!("algorithm should be a RSA in this example"),
    };

    let validation = {
        let mut validation = Validation::new(header.alg);
        validation.set_audience(&["https://takassh.shuttleapp.rs"]);
        validation.set_required_spec_claims(&["sub", "exp", "iss"]);
        validation
    };

    let decoded_token = decode::<Claims>(token, &decoding_key, &validation)?;

    Ok(decoded_token.claims)
}
