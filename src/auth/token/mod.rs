use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use rocket::http::Status;
use rocket::Request;
use rocket::request::{FromRequest, Outcome};
use crate::database::User;
use crate::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct Token {
    token: String,
    #[serde(skip)]
    pub claims: Claims
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Claims {
    exp: u128,
    pub iss: String,
    pub adm: bool,
    pub team: Option<String>
}

impl From<User> for Claims {
    fn from(user: User) -> Self {
        let now = current_time();
        let exp = now + 12 * 60 * 60 * 1000;
        let (adm, team, iss) = (user.adm, user.team, user.name);

        Claims {
            exp,
            iss,
            adm,
            team
        }
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for Token {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = request.cookies().get("Authenticate");

        match token {
            Some(token) => {
                let token = token.value();
                match validate(token) {
                    Ok(claims) => Outcome::Success(Token { token: token.to_owned(), claims }),
                    Err(_) => Outcome::Failure((Status::Forbidden, ()))
                }
            }
            None => Outcome::Failure((Status::Forbidden, ()))
        }
    }
}

pub fn issue(user: User) -> String {
    let claims = Claims::from(user);

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_secret().as_bytes())
    ).unwrap();

    format!("{{ \"token\":\"{}\"}}", token)
}

pub fn validate(token: &str) -> Result<Claims, ()> {
    let validation = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(get_secret().as_bytes()),
        &Validation::default()
    );

    validation.map_both(|claims| claims.claims, |_| ())
}

fn get_secret() -> String {
    from_config("jwt_secret")
}

fn current_time() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}