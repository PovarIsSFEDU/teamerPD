use rocket::Request;
use rocket::request::{FromRequest, Outcome};
use crate::auth::token;

pub struct Validator {
    pub validated: bool
}

#[async_trait]
impl<'r> FromRequest<'r> for Validator {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = request.headers().get_one("Authorization");

        match token {
            None => Outcome::Success(Validator {validated: false}),
            Some(token) => Outcome::Success(Validator {validated: token::validate(token)})
        }
    }
}