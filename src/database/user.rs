use crate::auth::RegistrationData;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    adm: bool,
    team: Option<String>,
    #[serde(flatten)]
    data: RegistrationData
}

impl User {
    pub fn adm(&self) -> bool {
        self.adm
    }

    pub fn team(&self) -> Option<String> {
        self.team.clone()
    }

    pub fn data(&self) -> &RegistrationData {
        &self.data
    }
}

impl From<RegistrationData> for User {
    fn from(data: RegistrationData) -> Self {
        let mut result = User {
            adm: false,
            team: None,
            data
        };
        result.data.hash();

        result
    }
}