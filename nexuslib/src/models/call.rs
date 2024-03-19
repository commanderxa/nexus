use serde::{Deserialize, Serialize};

pub mod media_call;

/// The structs that implement this one
/// can be inserted into the `CallRequest`
pub trait CallContent {}

#[derive(Debug, Serialize, Deserialize)]
/// Used to extract `CallType` since the
/// call content is not known at that time
pub struct EmptyCallBody {}

impl CallContent for EmptyCallBody {}
