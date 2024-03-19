use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::message::media::MediaType;

use super::FileContent;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The representation of an audio call
pub struct MediaFile {
    pub uuid: Uuid,
    pub len_bytes: usize,
    pub len_chunks: usize,
    pub name: String,
    pub media_type: MediaType,
    pub secret: bool,
    pub sender: Uuid,

    created_at: i64,
}

impl MediaFile {
    /// Creates new `MediaFile`
    pub fn new(
        uuid: Uuid,
        len_bytes: usize,
        len_chunks: usize,
        name: String,
        media_type: MediaType,
        secret: bool,
        sender: Uuid,
    ) -> Self {
        Self {
            uuid,
            len_bytes,
            len_chunks,
            name,
            media_type,
            secret,
            sender,
            created_at: Utc::now().timestamp(),
        }
    }

    /// Returns `timestamp` as `DateTime<Utc>` that
    /// specifies the time when this `MediaFile` was created
    pub fn get_created_at(&self) -> DateTime<Utc> {
        Utc.timestamp_opt(self.created_at, 0).unwrap()
    }

    /// Serializes an instance of `MediaFile` to `bytes`
    pub fn as_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    /// Deserializes an instance of `MediaFile` from `bytes`
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        bincode::deserialize(&bytes).unwrap()
    }
}

impl FileContent for MediaFile {
    fn get_type(&self) -> Option<MediaType> {
        Some(self.media_type)
    }
}
