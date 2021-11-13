use serde::{Serialize, Deserialize};
use crypto_hash::Algorithm;
use crate::auth::RegistrationData;

#[derive(Serialize, Deserialize)]
pub struct NewUser {
    pub verification_key: String,
    is_verified: bool,
    #[serde(flatten)]
    data: RegistrationData
}

impl From<RegistrationData> for NewUser {
    fn from(data: RegistrationData) -> Self {
        let mut key = data.login().clone();
        key.push_str(data.password().as_str());
        let key = crypto_hash::hex_digest(Algorithm::SHA256, key.as_bytes());

        let mut new_user = NewUser {
            verification_key: key,
            is_verified: false,
            data
        };

        new_user.data.hash();

        new_user
    }
}