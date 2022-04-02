use std::time::{SystemTime, UNIX_EPOCH};
use crypto_hash::Algorithm;

pub fn hash(data: &[u8]) -> String {
    crypto_hash::hex_digest(Algorithm::SHA256, data)
}

pub fn hash_unique(mut data: String) -> String {
    let date = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();
    let date = date.as_str();

    data.push_str(date);
    hash(data.as_bytes())
}

