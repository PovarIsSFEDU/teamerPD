use crypto_hash::Algorithm;

pub fn hash(data: &[u8]) -> String {
    crypto_hash::hex_digest(Algorithm::SHA256, data)
}