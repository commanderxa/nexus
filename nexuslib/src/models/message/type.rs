use serde_repr::{Deserialize_repr, Serialize_repr};
use strum_macros::Display;

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, Display)]
#[repr(u8)]
/// Type of a `Message`
///
/// Can be represented as u8 index
pub enum MessageType {
    Text,
    File,
}

impl MessageType {
    /// Returns u8 index of the `Role` entry
    pub fn get_index(&self) -> u8 {
        serde_json::to_string(self).unwrap().parse::<u8>().unwrap()
    }
}
