use serde::{Deserialize, Serialize};

use crate::models::message::MessageContent;
use crate::models::message::message::Message;

use super::Command;
use super::RequestBody;

#[derive(Serialize, Deserialize, Debug)]
/// Contains a `Message` of a particular type inside
pub struct MessageRequest<T: MessageContent> {
    pub message: Message<T>,
}

impl<T: MessageContent> MessageRequest<T> {
    pub fn new(message: Message<T>) -> Self {
        Self { message }
    }
}

impl<T: MessageContent> RequestBody for MessageRequest<T> {
    fn op(&self) -> Command {
        Command::Message
    }
}
