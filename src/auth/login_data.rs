use rocket::data::{FromData, Outcome, ToByteUnit};
use rocket::{Request, Data};
use serde::{Serialize, Deserialize};
use rocket::http::Status;

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginData {
    login: String,
    email: String,
    password: String
}

impl LoginData {
    pub fn new(login: String, email: String, password: String) -> Self { LoginData {login, email, password} }
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
            Outcome::Failure((Status::BadRequest, String::from("Error receiving login data")))
        }
    }
}