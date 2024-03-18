use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::request::sides::{RequestSides, RequestSidesOpt};

use super::{call_type::CallType, CallContent};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The representation of an audio call
pub struct AudioCall {
    pub uuid: Uuid,
    pub message: Vec<u8>,
    pub nonce: Vec<u8>,
    pub sides: RequestSides,
    pub peers: RequestSidesOpt,
    pub secret: bool,
    pub accepted: bool,

    created_at: i64,
}

impl AudioCall {
    /// Creates new `AudioCall`
    pub fn new(
        sender: Uuid,
        receiver: Uuid,
        message: Vec<u8>,
        nonce: Vec<u8>,
        accepted: bool,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            message,
            nonce,
            sides: RequestSides::new(sender, receiver),
            peers: RequestSidesOpt::new(),
            secret: false,
            accepted,
            created_at: Utc::now().timestamp(),
        }
    }

    /// Returns the Type of the Call
    pub fn get_type(&self) -> CallType {
        CallType::Audio
    }

    /// Returns `timestamp` as `DateTime<Utc>` that
    /// specifies the time when this `AudioCall` was created
    pub fn get_created_at(&self) -> DateTime<Utc> {
        Utc.timestamp_opt(self.created_at, 0).unwrap()
    }

    /// Serializes an instance of `AudioCall` to `bytes`
    pub fn as_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    /// Deserializes an instance of `AudioCall` from `bytes`
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        bincode::deserialize(&bytes).unwrap()
    }

    /// Returns the duration of the call in seconds
    pub fn duration(&self) -> i64 {
        match self.accepted {
            true => Utc::now().timestamp() - self.get_created_at().timestamp(),
            false => 0,
        }
    }
}

impl CallContent for AudioCall {
    fn get_type(&self) -> Option<CallType> {
        Some(CallType::Audio)
    }
}
