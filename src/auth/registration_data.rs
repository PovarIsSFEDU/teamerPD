use rocket::data::{FromData, Outcome};
use rocket::{Request, Data};
use serde::{Serialize, Deserialize};
use rocket::http::Status;
use crate::prelude::request;

#[derive(Clone, Serialize, Deserialize)]
pub struct RegistrationData {
    login: String,
    email: String,
    password: String
}

impl RegistrationData {
    pub fn login(&self) -> &String {
        &self.login
    }
    pub fn password(&self) -> &String {
        &self.password
    }
    pub fn email(&self) -> &String {
        &self.email
    }
    pub fn hash(&mut self) {
        self.password = bcrypt::hash(&self.password, 5).unwrap();
    }
}

#[async_trait]
impl<'r> FromData<'r> for RegistrationData {
    type Error = String;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let body = request::body_to_string(req, data).await;

        let result = serde_json::from_str::<RegistrationData>(body.as_str()).ok();

        if let Some(data) = result {
            Outcome::Success(data)
        } else {
            Outcome::Failure((Status::BadRequest, String::from("Error receiving login data")))
        }
    }
}