use std::convert::Infallible;

use serde::Serialize;
use warp::{hyper::StatusCode, Rejection, Reply};

use self::jwt::JWTError;

pub mod db;
pub mod jwt;

#[derive(Serialize)]
struct ErrorResponse {
    pub message: String,
    pub status: String,
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if let Some(e) = err.find::<JWTError>() {
        match e {
            JWTError::WrongCredentialsError => (StatusCode::FORBIDDEN, e.to_string()),
            JWTError::JWTTokenError => (StatusCode::UNAUTHORIZED, e.to_string()),
            JWTError::NoPermissionError => (StatusCode::UNAUTHORIZED, e.to_string()),
            JWTError::JWTTokenCreationError => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            _ => (StatusCode::BAD_REQUEST, e.to_string()),
        }
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        (
            StatusCode::METHOD_NOT_ALLOWED,
            "Method Not Allowed".to_string(),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    let response = warp::reply::json(&ErrorResponse {
        message: message,
        status: code.to_string(),
    });

    Ok(warp::reply::with_status(response, code))
}
