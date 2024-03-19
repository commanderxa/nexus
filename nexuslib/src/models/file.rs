use serde::{Deserialize, Serialize};

use super::message::media::MediaType;

pub mod media_file;

/// The structs that implement this one
/// can be inserted into the `FileRequest`
pub trait FileContent {
    fn get_type(&self) -> Option<MediaType>;
}

#[derive(Debug, Serialize, Deserialize)]
/// Used to extract `FileType` since the
/// call content is not known at that time
pub struct EmptyFileBody {}

impl FileContent for EmptyFileBody {
    fn get_type(&self) -> Option<MediaType> {
        None
    }
}
