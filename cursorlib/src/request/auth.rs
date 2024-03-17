use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
/// Authentication data
pub struct AuthRequest {
    pub username: String,
    pub password: String,
    pub meta: AuthRequestMeta,
}

#[derive(Serialize, Deserialize, Debug)]
/// Meta-Data for authentication
pub struct AuthRequestMeta {
    pub location: String,
    pub device_name: String,
    pub device_type: String,
    pub device_os: String,
}

#[derive(Serialize, Deserialize, Debug)]
/// Data to logout from a session
pub struct LogoutRequest {
    pub token: String,
}
