use std::collections::HashMap;

use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

pub struct ConnectionState {
    pub peers: HashMap<Uuid, HashMap<String, UnboundedSender<String>>>,
}

impl ConnectionState {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }
}
