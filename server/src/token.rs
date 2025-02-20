use serde::{Deserialize, Serialize};

use jsonwebtoken::{decode, errors::Result as JwtResult, Algorithm, DecodingKey, Validation};

#[derive(Debug, Serialize, Deserialize)]
enum Roles {
    Admin,
    User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: uuid::Uuid,
    pub iat: i64,
    pub exp: i64,
    pub roles: Vec<Roles>,
}

fn validate_token(token: &str, secret: &[u8]) -> JwtResult<TokenClaims> {
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<TokenClaims>(token, &DecodingKey::from_secret(secret), &validation)?;
    Ok(token_data.claims)
}
