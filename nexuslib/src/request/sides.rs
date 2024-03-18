use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// General structure that contains
/// sender and receiver of content
pub struct RequestSides {
    sender: Uuid,
    receiver: Uuid,
}

impl RequestSides {
    pub fn new(sender: Uuid, receiver: Uuid) -> Self {
        Self { sender, receiver }
    }

    pub fn get_sender(&self) -> Uuid {
        self.sender
    }

    pub fn get_receiver(&self) -> Uuid {
        self.receiver
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// General structure that contains
/// sender and receiver of content
pub struct RequestSidesOpt {
    sender: Option<Uuid>,
    receiver: Option<Uuid>,
}

impl RequestSidesOpt {
    pub fn new() -> Self {
        Self {
            sender: None,
            receiver: None,
        }
    }

    pub fn set_sender(&mut self, sender: Uuid) {
        self.sender = Some(sender);
    }

    pub fn set_receiver(&mut self, receiver: Uuid) {
        self.receiver = Some(receiver);
    }

    pub fn get_sender(&self) -> Option<Uuid> {
        self.sender
    }

    pub fn get_receiver(&self) -> Option<Uuid> {
        self.receiver
    }
}

impl Default for RequestSidesOpt {
    fn default() -> Self {
        Self::new()
    }
}
