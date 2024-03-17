use core::fmt;
use std::str::FromStr;

use serde_repr::{Deserialize_repr, Serialize_repr};

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Serialize_repr, Deserialize_repr)]
/// Role of a `User`
/// 
/// Can be represented as u8 index
pub enum Role {
    Admin,
    Moderator,
    User,
}

impl Role {
    /// Returns u8 index of the `Role` entry
    pub fn get_index(&self) -> u8 {
        serde_json::to_string(self).unwrap().parse::<u8>().unwrap()
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Moderator => write!(f, "moderator"),
            Role::User => write!(f, "user"),
        }
    }
}

impl FromStr for Role {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin" => Ok(Role::Admin),
            "moderator" => Ok(Role::Moderator),
            "user" => Ok(Role::User),
            _ => Err(()),
        }
    }
}
