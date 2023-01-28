use std::{collections::HashMap, net::SocketAddr};

use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

pub struct ConnectionState {
    pub peers: HashMap<Uuid, HashMap<Uuid, SessionSocket>>,
}

impl ConnectionState {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }
}

pub struct SessionSocket {
    pub socket_addr: SocketAddr,
    pub tcp_sender: UnboundedSender<String>,
}

impl SessionSocket {
    pub fn new(socket_addr: SocketAddr, tcp_sender: UnboundedSender<String>) -> Self {
        Self {
            socket_addr,
            tcp_sender,
        }
    }
}
