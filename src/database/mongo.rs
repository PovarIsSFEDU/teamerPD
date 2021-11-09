use mongodb::{Client, Collection};
use mongodb::bson::doc;
use crate::database::{RegisterResult, LoginResult, User, VerificationError, DatabaseError};
use crate::auth::{RegistrationData, LoginData};
use crate::prelude::MapBoth;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::marker::Send;
use crate::database::dbinsert::DbInsert;

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
        let db = self.client.database("user").collection::<DbInsert>("login");
        let user = User::from(data);
        let insert = DbInsert::from(user.clone());
        let result = db.insert_one(insert, None).await;

        match result {
            Ok(_) => Ok(user),
            Err(_) => Err(RegisterResult::Other)
        }
    }

    pub async fn validate_login(&self, data: LoginData) -> Result<User, LoginResult> {
        let found = self.get::<User>("login", data.login()).await;

        match found {
            Ok(Some(result)) => {
                let matches = bcrypt::verify(data.password(), result.data().password()).unwrap_or(false);

                match matches {
                    true => Ok(result),
                    false => Err(LoginResult::IncorrectPassword)
                }
            }

            Ok(None) => Err(LoginResult::NotExist),
            Err(_) => Err(LoginResult::Other)
        }
    }

    pub async fn verify_email(&self, key: String, login: String) -> Result<(), VerificationError> {
        let collection = self.client.database("user").collection::<LoginData>("login");

        let filter = doc! {"login": login, "verification_key": key, "is_verified": false};
        let modification = doc! {"$set": {"is_verified": true}, "$unset": {"verification_key": ""}};

        let result = collection.update_one(filter, modification, None).await;

        match result {
            Ok(result) if result.matched_count > 0 => Ok(()),
            Ok(_) => Err(VerificationError::AlreadyVerified),
            Err(_) => Err(VerificationError::Other)
        }
    }

    pub async fn get_verification_key(&self, login: String) -> Result<(String, String), DatabaseError> {
        #[derive(Deserialize)]
        struct VKey { #[serde(alias = "verification_key")] value: String, email: String }

        let key = self.get::<VKey>("login", &login).await;
        match key {
            Ok(Some(key)) => Ok((key.email, key.value)),
            Ok(None) => Err(DatabaseError::NotFound),
            Err(_) => Err(DatabaseError::Other)
        }
    }

    pub async fn set_recovery_key(&self, user: &RegistrationData, key: &str) -> Result<(), DatabaseError> {
        let collection = self.get_login_collection::<LoginData>();
        let filter = doc! {"login": user.login()};
        let modification = doc! {"$set": {"recovery_key": key}};
        let update_result = collection.update_one(filter, modification, None).await;

        update_result.map_both(|_| (), |_| DatabaseError::Other)
    }

    pub async fn validate_recovery(&self, key: String, user: LoginData) -> Result<(), DatabaseError> {
        let collection = self.get_login_collection::<LoginData>();
        let filter = doc! {"login": user.login(), "recovery_key": key};
        let found = collection.find_one(filter.clone(), None).await;
        match found {
            Ok(Some(_)) => {
                let new_pass = bcrypt::hash(user.password(), 5).unwrap();
                let modification = doc! {"$unset": {"recovery_key": ""}, "$set": {"password": new_pass}};
                let result = collection.update_one(filter, modification, None).await;

                result.map_both(|_| (), |_| DatabaseError::Other)
            }

            Ok(None) => Err(DatabaseError::NotFound),
            Err(_) => Err(DatabaseError::Other)
        }
    }

    pub async fn get<T>(&self, field: &str, value: &str) -> mongodb::error::Result<Option<T>>
        where T: DeserializeOwned + Unpin + Send + Sync
    {
        let db = self.get_login_collection::<T>();
        db.find_one(doc! {field: value.to_string()}, None).await
    }

    fn get_login_collection<T>(&self) -> Collection<T>
        where T: DeserializeOwned + Unpin + Send + Sync
    {
        self.client.database("user").collection::<T>("login")
    }
}