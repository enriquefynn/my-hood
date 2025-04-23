use std::{env, future::Future};

use crate::{oauth::get_token, user::model::User};
use anyhow::Error;
use axum::{
    extract::{FromRequest, FromRequestParts, Request},
    http::{request::Parts, HeaderMap},
    response::IntoResponse,
    Extension, Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;

use crate::DB;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Roles {
    Treasurer,
    Admin,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Option<uuid::Uuid>,
    pub email: Option<String>,
    pub exp: usize,
}

// A small extractor that pulls Claims from Request.extensions()
impl<S> FromRequest<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    fn from_request(
        req: Request,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let claims = req.extensions().get::<Claims>().cloned();
        async move {
            match claims {
                Some(claims) => Ok(claims),
                None => Err((StatusCode::UNAUTHORIZED, "Missing or invalid token")),
            }
        }
    }
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request_parts(parts, state).await.unwrap();
        let claims = extract_claims_from_request(&parts.headers, &cookies);
        match claims {
            Some(claims) => Ok(claims),
            None => Err((StatusCode::UNAUTHORIZED, "Missing or invalid token")),
        }
    }
}

pub fn extract_claims_from_request(headers: &HeaderMap, cookies: &Cookies) -> Option<Claims> {
    // First, try Authorization header
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(claims) = validate_token(token) {
                    return Some(claims);
                }
            }
        }
    }

    // If no header, fallback to auth_token cookie
    if let Some(cookie) = cookies.get("auth_token") {
        if let Ok(claims) = validate_token(cookie.value()) {
            return Some(claims);
        }
    }

    None
}

// Function to validate JWT token
fn validate_token(token: &str) -> Result<Claims, Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token: TokenData<Claims> = decode(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token.claims)
}

/// Request payload for logging in.
#[derive(Debug, Deserialize)]
pub struct LoginOrCreateRequest {
    email: String,
    password: String,
}

/// Response payload for login containing the token.
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    token: String,
}

pub fn get_token_exp() -> usize {
    Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize
}

pub async fn login_handler(
    Extension(db): Extension<DB>,
    Json(payload): Json<LoginOrCreateRequest>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let mut tx = db
        .begin()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB error"))?;
    let user = sqlx::query_as!(
        User,
        r#"SELECT * FROM "User" WHERE email = $1"#,
        payload.email
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid credentials"))?;

    // Create a token that expires in 24 hours.
    let exp = get_token_exp();

    let claims = match user {
        Some(user) => {
            let sub = Some(user.id);
            if let Some(user_password_hash) = user.password_hash {
                // Verify the password.
                let verify = bcrypt::verify(payload.password.clone(), &user_password_hash)
                    .map_err(|_| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Password verification failed",
                        )
                    })?;
                if verify == false {
                    return Err((StatusCode::UNAUTHORIZED, "Invalid credentials"));
                }
            }

            Claims {
                sub,
                email: user.email,
                exp,
            }
        }
        None => Claims {
            sub: None,
            email: None,
            exp,
        },
    };

    let token = get_token(claims.sub, claims.email.clone())?;
    return Ok(Json(LoginResponse { token }));
}
