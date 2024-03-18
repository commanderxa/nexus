use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{request::sides::RequestSides, utils::vec_to_string};

use self::{media::Media, r#type::MessageType, status::MessageStatus};

pub mod media;
pub mod status;
pub mod text;
pub mod r#type;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub media: Option<Media>,

    message_type: MessageType,
    created_at: i64,
    editead_at: Option<i64>,
}

impl<T: MessageContent> Message<T> {
    /// Creates a new `Message`
    pub fn new(content: T, nonce: Vec<u8>, sender: Uuid, receiver: Uuid) -> Self {
        let message_type: MessageType = content.get_type().unwrap();
        Self {
            uuid: Uuid::new_v4(),
            content,
            nonce: vec_to_string(nonce),
            sides: RequestSides::new(sender, receiver),
            status: MessageStatus::new(),
            ttl: None,
            secret: false,
            media: None,
            message_type,
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
        self.editead_at
            .map(|value| Utc.timestamp_opt(value, 0).unwrap())
    }

    /// Returns the type of the `Message`
    pub fn get_message_type(&self) -> MessageType {
        self.message_type
    }
}

/// The structs that implement this one
/// can be inserted into the `MessageRequest`
pub trait MessageContent {
    fn get_type(&self) -> Option<MessageType>;

    fn get_text(&self) -> Option<String>;
}

#[derive(Debug, Serialize, Deserialize)]
/// Used to extract `MessageType` since the
/// message content is not known at that time
pub struct EmptyMessageBody {}

impl MessageContent for EmptyMessageBody {
    fn get_type(&self) -> Option<MessageType> {
        None
    }

    fn get_text(&self) -> Option<String> {
        None
    }
}
