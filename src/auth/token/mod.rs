use std::fs;
use jsonwebtoken::{EncodingKey, Header};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use toml::Value;
use crate::database::User;

#[derive(Serialize)]
pub struct Token {
    token: String
}

#[derive(Clone, Serialize, Deserialize)]
struct Claims {
    exp: u128,
    iss: String,
    adm: bool,
    team: Option<String>
}

impl From<User> for Claims {
    fn from(user: User) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let exp = now + 12 * 60 * 60 * 1000;
        let (adm, team, iss) = (user.adm(), user.team(), user.data().login().clone());

        Claims {
            exp,
            iss,
            adm,
            team
        }
    }
}

pub fn issue(user: User) -> Token {
    let claims = Claims::from(user);

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_secret().as_bytes())
    ).unwrap();

    Token { token }
}

fn get_secret() -> String {
    let toml = fs::read_to_string("Config.toml").expect("Could not open toml");
    let value = toml.as_str().parse::<Value>().unwrap();

    value["jwt_secret"].as_str().unwrap().to_owned()
}