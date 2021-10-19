use rocket::data::{FromData, Outcome, ToByteUnit};
use rocket::{Request, Data};
use serde::{Serialize, Deserialize};
use rocket::http::Status;

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginData {
    login: String,
    password: String
}

impl LoginData {
    fn login(&self) -> &String {
        &self.login
    }

    fn password(&self) -> &String {
        &self.password
    }
}

#[async_trait]
impl<'o> FromData<'o> for LoginData {
    type Error = String;

    async fn from_data(req: &'o Request<'_>, data: Data<'o>) -> Outcome<'o, Self> {
        let length = req
            .headers()
            .get_one("Content-Length")
            .unwrap_or("2048")
            .parse::<i32>()
            .unwrap_or(2048)
            .bytes();

        let body = data
            .open(length)
            .into_string()
            .await
            .unwrap()
            .value;

        let result = serde_json::from_str::<LoginData>(body.as_str()).ok();

        if let Some(data) = result {
            Outcome::Success(data)
        } else {
            Outcome::Failure((Status::BadRequest, String::from("Incorrect login data")))
        }
    }
}