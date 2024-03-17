use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::models::calls::CallContent;

use super::Command;
use super::IndexToken;
use super::RequestBody;

#[derive(Debug, Serialize, Deserialize)]
/// Contains a `Call` of a particular type inside
pub struct CallRequest<T>
where
    T: CallContent,
{
    pub call: T,
    pub index: IndexToken,
    pub created_at: i64,
}

impl<T: CallContent> CallRequest<T> {
    pub fn new(call: T, index: IndexToken) -> Self {
        Self {
            call: call,
            index: index,
            created_at: Utc::now().timestamp(),
        }
    }
}

impl<T: CallContent> RequestBody for CallRequest<T> {
    fn op(&self) -> Command {
        Command::AudioCall
    }
}
