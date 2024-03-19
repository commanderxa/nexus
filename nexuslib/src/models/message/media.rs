use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Media {
    pub attachments: Vec<MediaAttachment>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// The representation of a File that is a `Message`
pub struct MediaAttachment {
    pub uuid: Uuid,
    pub name: String,
    pub path: String,
    pub media_type: MediaType,
}

impl MediaAttachment {
    /// Creates new `MediaAttachment`
    pub fn new(uuid: Uuid, name: &str, path: &str, media_type: MediaType) -> Self {
        Self {
            uuid,
            name: name.to_owned(),
            path: path.to_owned(),
            media_type,
        }
    }

    pub fn get_type(&self) -> MediaType {
        self.media_type
    }
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, Display, EnumIter)]
#[repr(u8)]
pub enum MediaType {
    Audio,
    File,
    Image,
    Video,
}

impl MediaType {
    /// Returns u8 index of the `Role` entry
    pub fn get_index(&self) -> u8 {
        serde_json::to_string(self).unwrap().parse::<u8>().unwrap()
    }

    /// Returns a vector of enum variants of `MediaType`
    pub fn str_variants_vec() -> Vec<String> {
        let mut arr = vec![];
        for value in MediaType::iter() {
            arr.push(value.to_string().to_lowercase());
        }
        arr
    }
}
