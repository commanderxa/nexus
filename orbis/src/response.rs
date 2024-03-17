use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub mod auth;

#[derive(Debug, Serialize, Deserialize)]
/// Send response from server to client using websockets
pub struct Response<T>
where
    T: ResponseBody,
{
    pub status: ResponseStatus,
    pub content: T,
}

impl<T: ResponseBody> Response<T> {
    pub fn new(status: ResponseStatus, content: T) -> Self {
        Self { status, content }
    }
}

/// `Response` content has to implement this trait
pub trait ResponseBody {}

impl ResponseBody for String {}

#[derive(Debug, Serialize, Deserialize)]
/// Status of a `Response`. Either `Ok` or `Err`
pub enum ResponseStatus {
    Ok,
    Err,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Serialize_repr, Deserialize_repr)]
pub enum ResponseStatusCode {
    ConnectionEstablished = 202,
}

impl ResponseBody for ResponseStatusCode {}
