use serde::{Deserialize, Serialize};

use super::{msg_type::MessageType, MessageContent};

#[derive(Debug, Serialize, Deserialize)]
/// The representation of a File that is a `Message`
pub struct FileMessage {
    pub text: String,
    pub filename: String,
    pub file: Vec<u8>,
}

impl FileMessage {
    /// Creates new `FileMessage`
    pub fn new(text: &str, filename: &str, file: Vec<u8>) -> Self {
        Self {
            text: text.to_owned(),
            filename: filename.to_owned(),
            file,
        }
    }
}

impl MessageContent for FileMessage {
    fn get_type(&self) -> Option<MessageType> {
        Some(MessageType::File)
    }

    fn get_text(&self) -> Option<String> {
        Some(self.text.to_owned())
    }
}
