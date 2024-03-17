use std::{str::FromStr, sync::Arc};

use jsonwebtoken::{decode, Algorithm, DecodingKey, TokenData, Validation};
use cursorlib::{
    models::user::role::Role,
    request::auth::{AuthRequest, LogoutRequest},
};
use scylla::Session;
use tokio::sync::Mutex;
use warp::{http::HeaderValue, hyper::HeaderMap, reject, Filter, Rejection};

use crate::{
    errors::jwt::JWTError,
    handlers::{self, auth::validate_session},
    jwt::{jwt_from_header, Claims},
};

use super::with_session;

// All auth routes
pub fn auth(
    session: Arc<Mutex<Session>>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("auth").and(
        login(session.clone())
            .or(register(session.clone()))
            .or(logout(session.clone())),
    )
}

/// POST /auth/login
pub fn login(
    session: Arc<Mutex<Session>>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("login")
        .and(warp::post())
        .and(with_session(session))
        .and(warp::body::json())
        .and_then(handlers::auth::login)
}

/// POST /auth/register
pub fn register(
    session: Arc<Mutex<Session>>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("register")
        .and(warp::post())
        .and(with_session(session))
        .and(json_body())
        .and_then(handlers::auth::register)
}

pub fn logout(
    session: Arc<Mutex<Session>>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("logout")
        .and(warp::post())
        .and(with_session(session))
        .and(json_body_logout())
        .and_then(handlers::auth::logout)
}

fn json_body() -> impl Filter<Extract = (AuthRequest,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn json_body_logout() -> impl Filter<Extract = (LogoutRequest,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

pub async fn authorize(
    (role, headers): (Role, HeaderMap<HeaderValue>),
    session: Arc<Mutex<Session>>,
) -> Result<(), Rejection> {
    match jwt_from_header(&headers) {
        Ok(token) => {
            let decoded = check_token(session, &token)
                .await
                .map_err(|_| reject::custom(JWTError::JWTTokenError))?;

            if ![Role::Admin, Role::Moderator]
                .contains(&Role::from_str(&decoded.claims.role).unwrap())
                && [Role::Admin, Role::Moderator].contains(&role)
            {
                return Err(reject::custom(JWTError::NoPermissionError));
            }

            Ok(())
        }
        Err(e) => return Err(reject::custom(e)),
    }
}

pub async fn check_token(
    session: Arc<Mutex<Session>>,
    token: &str,
) -> Result<TokenData<Claims>, JWTError> {
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(b"123"),
        &Validation::new(Algorithm::HS512),
    )
    .map_err(|_| JWTError::JWTTokenError)?;

    // Check tokens
    let validated = validate_session(session, &token).await;
    if validated.is_err() {
        return Err(JWTError::JWTTokenError);
    }
    Ok(decoded)
}
