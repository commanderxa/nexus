use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::models::file::media_file::MediaFile;

use super::Command;
use super::RequestBody;

#[derive(Debug, Serialize, Deserialize)]
/// Contains a `File` of a particular type inside
pub struct FileRequest {
    pub file: MediaFile,
    pub created_at: i64,
}

impl FileRequest {
    pub fn new(file: MediaFile) -> Self {
        Self {
            file,
            created_at: Utc::now().timestamp(),
        }
    }
}

impl RequestBody for FileRequest {
    fn op(&self) -> Command {
        Command::File
    }
}
