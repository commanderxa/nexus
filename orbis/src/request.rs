use serde::{Deserialize, Serialize};

use crate::models::command::Command;

pub mod auth;
pub mod call;
pub mod message;
pub mod sides;

#[derive(Debug, Serialize, Deserialize)]
/// `Request` is used for communication in a websockets session
pub struct Request<T>
where
    T: RequestBody,
{
    pub command: Command,
    pub body: T,
    pub token: String,
}

impl<T: RequestBody> Request<T> {
    pub fn new(command: Command, body: T, token: String) -> Self {
        Self {
            command,
            body,
            token,
        }
    }
}

/// Indicates that the type can be encapsulated 
/// into the `Request`
pub trait RequestBody {
    fn op(&self) -> Command;
}

#[derive(Debug, Serialize, Deserialize)]
/// Used to extract the request command since 
/// the content is not yet know at that stage
pub struct EmptyRequestBody {}

impl RequestBody for EmptyRequestBody {
    fn op(&self) -> Command {
        todo!();
    }
}

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
/// `IndexToken` needed to mark the events
pub enum IndexToken {
    Start,
    Accept,
    Accepted,
    End,
}
