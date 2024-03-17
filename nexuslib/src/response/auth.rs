use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
/// Authentication response
pub struct AuthResponse {
    pub uuid: Uuid,
    pub token: String,
}

impl AuthResponse {
    /// Creates new AuthResponse instance
    pub fn new(uuid: Uuid, token: String) -> Self {
        Self { uuid, token }
    }
}
