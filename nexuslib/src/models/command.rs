use serde_repr::{Deserialize_repr, Serialize_repr};
use strum_macros::Display;

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, Display)]
#[repr(u8)]
/// Tells the server what function to call
pub enum Command {
    Message,
    Call,
    File,
}
