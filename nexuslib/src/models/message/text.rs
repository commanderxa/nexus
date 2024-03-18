use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::{msg_type::MessageType, MessageContent};

#[derive(Debug, Serialize, Deserialize)]
/// Text content of a `Message`
pub struct TextMessage {
    pub text: String,
}

impl TextMessage {
    /// Creates a new `TextMessage`
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
        }
    }
}

impl MessageContent for TextMessage {
    fn get_type(&self) -> Option<MessageType> {
        Some(MessageType::Text)
    }

    fn get_text(&self) -> Option<String> {
        Some(self.text.to_owned())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NotStringError;

impl FromStr for TextMessage {
    type Err = NotStringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let text_message: Self = TextMessage { text: s.to_owned() };
        let text = text_message;
        Ok(text)
    }
}
