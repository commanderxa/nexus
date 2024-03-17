use sha3::{Digest, Sha3_256};

/// Returns the hash of a text
pub fn get_hash(text: &str) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(text);
    hex::encode(hasher.finalize())
}
