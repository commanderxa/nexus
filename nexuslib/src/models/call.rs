use serde::{Deserialize, Serialize};

use self::call_type::CallType;

pub mod media_call;
pub mod call_type;

/// The structs that implement this one 
/// can be inserted into the `CallRequest`
pub trait CallContent {
    fn get_type(&self) -> Option<CallType>;
}

#[derive(Debug, Serialize, Deserialize)]
/// Used to extract `CallType` since the 
/// call content is not known at that time
pub struct EmptyCallBody {}

impl CallContent for EmptyCallBody {
    fn get_type(&self) -> Option<CallType> {
        None
    }
}
