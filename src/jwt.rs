use core::fmt;

use chrono::Duration;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    LOCALHOST,
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::LOCALHOST => write!(f, "LOCALHOST"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub token_version: u32,
    pub issuer: Service, // the application that signed the token
    pub exp: i64, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    pub iat: i64, // Optional. Issued at (as UTC timestamp)
}

pub fn sign(
    user_id: &str,
    token_version: u32,
    service: Service,
    jwt_secret: &str,
) -> Result<String, AppError> {
    let header = Header::new(Algorithm::HS512);
    let iat = chrono::Utc::now();
    let exp = iat + Duration::days(30);
    let claims = Claims {
        user_id: user_id.to_string(),
        token_version,
        issuer: service,
        exp: exp.timestamp(),
        iat: iat.timestamp(),
    };
    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(AppError::internal_server_error)
}

pub fn verify(token: &str, jwt_secret: &str) -> Result<Claims, AppError> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::new(Algorithm::HS512),
    )
    .map_err(AppError::internal_server_error)?;
    Ok(data.claims)
}
