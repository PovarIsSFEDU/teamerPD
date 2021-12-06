use crate::auth::RegistrationData;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub team: Option<String>,
    pub photo: Option<String>,
    pub resume: Option<String>,
    pub adm: bool,
    pub email: String
}


impl User {
    pub fn from(data: &RegistrationData) -> Self {
        User {
            name: data.login().clone(),
            team: None,
            photo: None,
            resume: None,
            adm: false,
            email: data.email().to_owned()
        }
    }
}