use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use cursorlib::models::user::role::Role;
use serde::{Deserialize, Serialize};
use warp::{
    http::HeaderValue,
    hyper::{header::AUTHORIZATION, HeaderMap},
};

use crate::errors::jwt::JWTError;

const BEARER: &str = "Bearer ";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

/// Creates JWT
pub fn generate_jwt(uid: &str, role: Role) -> Result<String, JWTError> {
    let seconds = 60 * 60 * 24 * 365;
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(seconds))
        .expect("Invalid timestamp")
        .timestamp();

    // let mut csprng = OsRng {};
    // let keypair: Keypair = Keypair::generate(&mut csprng);
    // let public_key: PublicKey = keypair.public;

    let claims = Claims {
        sub: uid.to_owned(),
        role: role.to_string().to_owned(),
        exp: expiration as usize,
    };

    let header = Header::new(Algorithm::HS512);

    encode(&header, &claims, &EncodingKey::from_secret(b"123"))
        .map_err(|_| JWTError::JWTTokenCreationError)
}

/// Extracts JWT from Header
pub fn jwt_from_header(headers: &HeaderMap<HeaderValue>) -> Result<String, JWTError> {
    let header = match headers.get(AUTHORIZATION) {
        Some(v) => v,
        None => return Err(JWTError::NoAuthHeaderError),
    };

    let auth_header = match std::str::from_utf8(header.as_bytes()) {
        Ok(v) => v,
        Err(_) => return Err(JWTError::NoAuthHeaderError),
    };

    if !auth_header.starts_with(BEARER) {
        return Err(JWTError::InvalidAuthHeaderError);
    }

    Ok(auth_header.trim_start_matches(BEARER).to_owned())
}
