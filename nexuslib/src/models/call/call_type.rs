use serde_repr::{Deserialize_repr, Serialize_repr};
use strum_macros::Display;

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, Display)]
#[repr(u8)]
/// The Type of a `Call`
/// 
/// Can be represented as u8 index
pub enum CallType {
    Audio,
    Video,
}

impl CallType {
    /// Returns u8 index of the `CallType` entry
    pub fn get_index(&self) -> u8 {
        serde_json::to_string(self).unwrap().parse::<u8>().unwrap()
    }
}
