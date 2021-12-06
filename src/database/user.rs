use crate::auth::RegistrationData;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct User {
    pub name: String,
    pub team: Option<String>,
    pub photo: Option<String>,
    pub resume: Option<String>,
    pub adm: bool,
}


impl User {
    pub fn from(data: &RegistrationData) -> Self {
        User {
            name: data.login().clone(),
            team: None,
            photo: None,
            resume: None,
            adm: false,
        }
    }
}