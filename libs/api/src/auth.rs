use std::collections::HashMap;

use anyhow::Context;
use axum::{
    extract::Request,
    http::{self},
    middleware::Next,
    response::Response,
};
use tokio::sync::OnceCell;

use crate::{ApiError, ACCEPT_USER, JWKS_URL};

use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet},
    DecodingKey, Validation,
};

static JWKS: OnceCell<JwkSet> = OnceCell::const_new();

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

pub async fn auth(req: Request, next: Next) -> Result<Response, ApiError> {
    let auth_error =
        Err(ApiError::AuthError("failed to authorization".to_string()));

    let jwks = match JWKS.get() {
        Some(jwks) => jwks.clone(),
        None => {
            let Ok(jwks) = get_jwks().await else {
                return auth_error;
            };
            JWKS.set(jwks.clone()).unwrap();
            jwks
        }
    };

    let Ok(token) = get_authorization_header(&req) else {
        return auth_error;
    };
    let Ok(header) = decode_header(&token) else {
        return auth_error;
    };

    let Some(kid) = header.kid else {
        return auth_error;
    };

    let Some(jwk) = jwks.find(&kid) else {
        return auth_error;
    };

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

    let result = decode::<HashMap<String, serde_json::Value>>(
        &token,
        &decoding_key,
        &validation,
    );
    let Ok(decoded_token) = result else {
        return auth_error;
    };

    if ACCEPT_USER.get().unwrap()
        == decoded_token.claims["sub"].as_str().unwrap()
    {
        return Ok(next.run(req).await);
    }

    auth_error
}
