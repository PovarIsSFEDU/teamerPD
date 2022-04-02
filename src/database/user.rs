use rocket::data::{FromData, Outcome};
use rocket::{Data, Request};
use rocket::http::Status;
use crate::auth::RegistrationData;
use serde::{Serialize, Deserialize};
use crate::request;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(default)]
    pub login: String,
    #[serde(default)]
    pub name: String,
    pub surname: String,
    pub team: Option<String>,
    pub photo: Option<String>,
    pub resume: Option<String>,
    #[serde(default)]
    pub adm: bool,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub competences: Vec<String>
}


impl User {
    pub fn from(data: &RegistrationData) -> Self {
        User {
            login: data.login().clone(),
            name: String::from(""),
            surname: String::from(""),
            team: None,
            photo: None,
            resume: None,
            adm: false,
            email: data.email().to_owned(),
            competences: vec![]
        }
    }
}

#[async_trait]
impl<'r> FromData<'r> for User {
    type Error = ();

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let body = request::body_to_string(req, data).await;
        let result = serde_json::from_str::<Self>(body.as_str());
        match result {
            Ok(user) =>  Outcome::Success(user),
            Err(_) => Outcome::Failure((Status::InternalServerError, ()))
        }
    }
}