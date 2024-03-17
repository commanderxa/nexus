use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
/// User's Settings
pub struct UserSettings {
    pub language: String,
    pub theme: String,
}
