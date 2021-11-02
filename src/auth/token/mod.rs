use std::fs;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use toml::Value;
use crate::database::User;

#[derive(Serialize, Deserialize)]
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
        let now = current_time();
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

pub fn validate(token: &str) -> bool {
    let validation = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(get_secret().as_bytes()),
        &Validation::default()
    );

    match validation {
        Ok(result) => current_time() < result.claims.exp,
        Err(_) => false
    }
}

fn get_secret() -> String {
    let toml = fs::read_to_string("Config.toml").expect("Could not open toml");
    let value = toml.as_str().parse::<Value>().unwrap();

    value["jwt_secret"].as_str().unwrap().to_owned()
}

fn current_time() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}