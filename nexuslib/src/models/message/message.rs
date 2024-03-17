use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{utils::vec_to_string, request::sides::RequestSides};

use super::{msg_type::MessageType, status::MessageStatus, MessageContent};

#[derive(Debug, Serialize, Deserialize)]
/// The message
pub struct Message<T>
where
    T: MessageContent,
{
    pub uuid: Uuid,
    pub content: T,
    nonce: String,
    pub sides: RequestSides,
    pub status: MessageStatus,
    pub ttl: Option<i64>,
    pub secret: bool,

    msg_type: MessageType,
    created_at: i64,
    editead_at: Option<i64>,
}

impl<T: MessageContent> Message<T> {
    /// Creates a new `Message`
    pub fn new(content: T, nonce: Vec<u8>, sender: Uuid, receiver: Uuid) -> Self {
        let msg_type: MessageType = content.get_type().unwrap();
        Self {
            uuid: Uuid::new_v4(),
            content: content,
            nonce: vec_to_string(nonce),
            sides: RequestSides::new(sender, receiver),
            status: MessageStatus::new(),
            ttl: None,
            secret: false,
            msg_type: msg_type,
            created_at: Utc::now().timestamp(),
            editead_at: None,
        }
    }

    /// Returns nonce of the `Message`
    pub fn get_nonce(&self) -> String {
        self.nonce.to_owned()
    }

    /// Returns `timestamp` as `DateTime<Utc>` that 
    /// specifies the time when this `Message` was created
    pub fn get_created_at(&self) -> DateTime<Utc> {
        Utc.timestamp_opt(self.created_at, 0).unwrap()
    }

    /// Returns `timestamp` as `Option<DateTime<Utc>>` that 
    /// specifies the time when this `Message` was edited (if it was)
    pub fn get_edited_at(&self) -> Option<DateTime<Utc>> {
        match self.editead_at {
            Some(value) => Some(Utc.timestamp_opt(value, 0).unwrap()),
            None => None,
        }
    }

    /// Returns the type of the `Message`
    pub fn get_msg_type(&self) -> MessageType {
        self.msg_type
    }
}
