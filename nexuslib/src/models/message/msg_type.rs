use core::fmt;

use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
/// Type of a `Message`
///
/// Can be represented as u8 index
pub enum MessageType {
    Text,
    Audio,
    Image,
    Video,
    File,
}

impl MessageType {
    /// Returns u8 index of the `Role` entry
    pub fn get_index(&self) -> u8 {
        serde_json::to_string(self).unwrap().parse::<u8>().unwrap()
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
