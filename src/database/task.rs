use rocket::data::{FromData, Outcome};
use rocket::{Data, Request};
use rocket::http::Status;
use crate::request;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub name: String,
    pub description: String,
    pub status: TaskStatus,
    pub assignee: Option<String>,
    pub priority: Priority,
    pub expiration: String
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TaskStatus {
    Open,
    InProgress,
    Complete
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Priority {
    Urgent,
    Normal,
    Secondary,
    Minor
}

#[async_trait]
impl<'r> FromData<'r> for Task {
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