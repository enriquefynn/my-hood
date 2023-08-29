use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: uuid::Uuid,
    pub iat: usize,
    pub exp: usize,
}
