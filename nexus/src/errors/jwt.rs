use thiserror::Error;
use warp::reject::Reject;

#[allow(unused)]
#[derive(Debug, Error)]
pub enum JWTError {
    #[error("Wrong Credentials")]
    WrongCredentials,
    #[error("JWT is not valid")]
    JWTToken,
    #[error("JWT creation error")]
    JWTTokenCreation,
    #[error("No Auth Header")]
    NoAuthHeader,
    #[error("Invalid Auth Header")]
    InvalidAuthHeader,
    #[error("No Permission")]
    NoPermission,
}

impl Reject for JWTError {}
