use serde::{Serialize, Deserialize};
use crate::database::User;
use crypto_hash::Algorithm;

#[derive(Serialize, Deserialize)]
pub struct DbInsert {
    pub verification_key: String,
    is_verified: bool,
    #[serde(flatten)]
    user: User
}

impl From<User> for DbInsert {
    fn from(user: User) -> Self {
        let mut key = user.data().login().clone();
        key.push_str(user.data().password().as_str());
        let key = crypto_hash::hex_digest(Algorithm::SHA256, key.as_bytes());

        DbInsert {
            verification_key: key,
            is_verified: false,
            user
        }
    }
}