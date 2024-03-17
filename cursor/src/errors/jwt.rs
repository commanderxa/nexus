use thiserror::Error;
use warp::reject::Reject;

#[allow(unused)]
#[derive(Debug, Error)]
pub enum JWTError {
    #[error("Wrong Credentials")]
    WrongCredentialsError,
    #[error("JWT is not valid")]
    JWTTokenError,
    #[error("JWT creation Error")]
    JWTTokenCreationError,
    #[error("No Auth Header")]
    NoAuthHeaderError,
    #[error("Invalid Auth Header")]
    InvalidAuthHeaderError,
    #[error("No Permission")]
    NoPermissionError,
}

impl Reject for JWTError {}
