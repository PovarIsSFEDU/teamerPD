use rocket::data::{FromData, Outcome};
use rocket::{Data, Request};
use rocket::http::Status;
use serde::{Serialize, Deserialize};
use crate::request;

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Team {
    pub name: String,
    pub logo: Option<String>,
    pub captain: String,
    pub members: Vec<String>,
    #[serde(default)]
    pub short_bio: String,
    #[serde(default)]
    pub long_bio: String,
    #[serde(default)]
    pub competences: Vec<String>,
}

#[async_trait]
impl<'r> FromData<'r> for Team {
    type Error = ();

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let body = request::body_to_string(req, data).await;
        let result = serde_json::from_str::<Self>(body.as_str());
        match result {
            Ok(team) => Outcome::Success(team),
            Err(_) => Outcome::Failure((Status::InternalServerError, ()))
        }
    }
}