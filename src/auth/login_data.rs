use rocket::data::{FromData, Outcome};
use rocket::{Data, Request};
use serde::{Serialize, Deserialize};
use rocket::http::Status;
use crate::prelude::request;

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginData {
    login: String,
    password: String
}

impl LoginData {
    pub fn login(&self) -> &String {
        &self.login
    }

    pub fn password(&self) -> &String {
        &self.password
    }
}

#[async_trait]
impl<'r> FromData<'r> for LoginData {
    type Error = String;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let body = request::body_to_string(req, data).await;

        let result = serde_json::from_str::<LoginData>(body.as_str()).ok();

        if let Some(data) = result {
            Outcome::Success(data)
        } else {
            Outcome::Failure((Status::BadRequest, String::from("Error receiving login data")))
        }
    }
}