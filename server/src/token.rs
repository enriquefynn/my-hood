use std::{env, future::Future};

use crate::user::model::User;
use anyhow::Error;
use axum::{
    extract::{FromRequest, FromRequestParts, Request},
    http::{request::Parts, HeaderMap},
    response::IntoResponse,
    Extension, Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::DB;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Roles {
    GlobalAdmin,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Option<uuid::Uuid>,
    pub email: Option<String>,
    pub exp: usize,
    pub roles: Vec<Roles>,
}

impl Claims {
    pub fn is_global_admin(&self) -> bool {
        self.roles.contains(&Roles::GlobalAdmin)
    }
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

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = extract_claims_from_header(&parts.headers);
        match claims {
            Some(claims) => Ok(claims),
            None => Err((StatusCode::UNAUTHORIZED, "Missing or invalid token")),
        }
    }
}

pub fn extract_claims_from_header(header_map: &HeaderMap) -> Option<Claims> {
    let auth_header = header_map.get("Authorization")?;
    let auth_str = auth_header.to_str().ok()?;
    let token = auth_str.strip_prefix("Bearer ")?;
    match validate_token(token) {
        Ok(token_claims) => Some(token_claims),
        Err(_) => None,
    }
}

// Function to validate JWT token
fn validate_token(token: &str) -> Result<Claims, Error> {
    let secret = env::var("SECRET").expect("SECRET must be set");
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
    let secret = env::var("SECRET").expect("SECRET must be set");

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

            //Check if the user is a global admin
            let global_admin =
                sqlx::query!(r#"SELECT * FROM "GlobalAdmin" WHERE user_id = $1"#, user.id)
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB error"))?;
            let mut roles = vec![];
            if global_admin.is_some() {
                roles.push(Roles::GlobalAdmin);
            }
            Claims {
                sub,
                email: user.email,
                exp,
                roles,
            }
        }
        None => Claims {
            sub: None,
            email: None,
            exp,
            roles: vec![],
        },
    };

    // Encode the token using the secret key.
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Token creation failed"))?;

    return Ok(Json(LoginResponse { token }));
}

// TODO: Send email with verification code etc.
pub async fn create_user(
    Extension(_db): Extension<DB>,
    Json(_payload): Json<LoginOrCreateRequest>,
) {
}
