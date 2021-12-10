use rocket::data::{FromData, Outcome};
use rocket::{Data, Request};
use crate::auth::RegistrationData;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(default)]
    pub name: String,
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
            name: data.login().clone(),
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
        todo!()
    }
}