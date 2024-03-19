use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
/// `CallToken` needed to mark the events
pub enum IndexToken {
    Start,
    Accept,
    Accepted,
    End,
}
