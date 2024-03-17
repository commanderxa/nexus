use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
/// Status of a `Message`
pub struct MessageStatus {
    sent: bool,
    read: bool,
    edited: bool,
}

impl MessageStatus {
    pub fn new() -> Self {
        Self {
            sent: false,
            read: false,
            edited: false,
        }
    }

    pub fn get_sent(&self) -> bool {
        self.sent
    }

    pub fn get_read(&self) -> bool {
        self.read
    }

    pub fn get_edited(&self) -> bool {
        self.edited
    }

    pub fn set_sent(&mut self) {
        if self.sent {
            return;
        }
        self.sent = true;
    }

    pub fn set_read(&mut self) {
        if self.read {
            return;
        }
        self.read = true;
    }

    pub fn set_edited(&mut self) {
        if self.edited {
            return;
        }
        self.edited = true;
    }
}
