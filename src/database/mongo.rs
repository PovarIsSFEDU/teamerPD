use mongodb::Client;
use mongodb::bson::doc;
use crate::database::{RegisterResult, LoginResult, User};
use crate::auth::{RegistrationData, LoginData};
use serde::de::DeserializeOwned;
use std::marker::Send;

pub struct MongoDriver {
    client: Client
}

impl MongoDriver {
    pub fn new(client: Client) -> Self {
        MongoDriver {
            client
        }
    }

    pub async fn validate_registration(&self, data: &RegistrationData) -> RegisterResult {
        let found = self.get::<RegistrationData>("login", data.login()).await;

        match found {
            Ok(None) => {
                let email = self.get::<RegistrationData>("email", data.email()).await;
                match email {
                    Ok(None) => RegisterResult::Ok,
                    Ok(Some(_)) => RegisterResult::Exists,
                    Err(_) => RegisterResult::Other
                }
            }

            Ok(Some(_)) => RegisterResult::Exists,
            Err(_) => RegisterResult::Other
        }
    }

    pub async fn register(&self, data: RegistrationData) -> Result<User, RegisterResult> {
        let db = self.client.database("user").collection::<User>("login");
        let user = User::from(data);
        let result = db.insert_one(user.clone(), None).await;

        match result {
            Ok(_) => Ok(user),
            Err(_) => Err(RegisterResult::Other)
        }
    }

    pub async fn validate_login(&self, data: &LoginData) -> Result<User, LoginResult> {
        let found = self.get::<User>("login", data.login()).await;

        match found {
            Ok(Some(result)) => {
                let matches = bcrypt::verify(data.password(), result.data().password()).unwrap_or(false);

                if matches {
                    Ok(result)
                } else {
                    Err(LoginResult::IncorrectPassword)
                }
            }

            Ok(None) => Err(LoginResult::NotExist),
            Err(_) => Err(LoginResult::Other)
        }
    }

    async fn get<T>(&self, field: &str, value: &String) -> mongodb::error::Result<Option<T>>
        where T: DeserializeOwned + Unpin + Send + Sync
    {
        let db = self.client.database("user").collection::<T>("login");
        db.find_one(doc! {field: value}, None).await
    }
}