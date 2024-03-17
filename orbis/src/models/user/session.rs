use uuid::Uuid;

/// The representation of the user's session
pub struct UserSession {
    pub user: Uuid,
    pub jwt: String,
    pub location: String,
    pub device: String,
    pub device_name: String,
    pub device_os: String,
    pub created_at: i64,
}
