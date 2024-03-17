use serde::{Deserialize, Serialize};

use self::msg_type::MessageType;

pub mod file;
pub mod message;
pub mod msg_type;
pub mod status;
pub mod text;

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
