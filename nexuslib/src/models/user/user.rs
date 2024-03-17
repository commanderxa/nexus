use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::utils::{string_to_vec, vec_to_string};

use super::role::Role;

#[derive(Debug, Serialize, Deserialize, Clone)]
/// The representation of a user
pub struct User {
    pub uuid: Uuid,
    pub username: String,
    pub password: String,
    pub role: Role,
    pub public_key: String,
    // pub photo: Vec<u8>,
    // pub settings: UserSettings,
    pub created_at: i64,
}

impl User {
    /// Creates a new `User`
    pub fn new(username: &str, password: &str, role: Option<Role>) -> (Self, [u8; 32]) {
        // init necessary values
        let user_uuid = Uuid::new_v4();
        let secret = StaticSecret::new(rand_core::OsRng);
        let public_key = PublicKey::from(&secret).to_bytes();

        (
            Self {
                uuid: user_uuid,
                username: username.to_owned(),
                password: password.to_owned(),
                role: role.unwrap_or(Role::User),
                public_key: vec_to_string(public_key.to_vec()),
                created_at: Utc::now().timestamp(),
            },
            secret.to_bytes(),
        )
    }

    /// Returns the user's public key as a `Vec<u8>`
    pub fn public_key(&self) -> Vec<u8> {
        string_to_vec(self.public_key.clone())
    }

    /// Returns the user's public key as a `String`
    pub fn public_key_str(&self) -> String {
        self.public_key.clone()
    }
}

impl ToString for User {
    fn to_string(&self) -> String {
        format!(
            "User\n\t- username: {0}\n\t- password: {1}\n\t- role: {2}",
            self.username,
            self.password,
            self.role.to_string()
        )
    }
}
