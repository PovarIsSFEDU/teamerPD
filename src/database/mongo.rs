use mongodb::Client;
use mongodb::bson::doc;
use crate::database::{DatabaseDriver, RegisterResult, LoginResult};
use crate::auth::LoginData;
use mongodb::error::{Result, Error};
use mongodb::results::InsertOneResult;
use bcrypt::BcryptError;

pub struct MongoDriver {
    client: Client
}

impl MongoDriver {
    pub fn new(client: Client) -> Self {
        MongoDriver {
            client
        }
    }

    async fn get_user(&self, field: &str, value: &String) -> Result<Option<LoginData>> {
        let db = self.client.database("user").collection::<LoginData>("login");
        db.find_one(doc! {field: value}, None).await
    }
}

#[async_trait]
impl DatabaseDriver for MongoDriver {
    async fn validate_registration(&self, data: &LoginData) -> RegisterResult {
        let found = self.get_user("login", data.login()).await;

        match found {
            Ok(None) => {
                let email = self.get_user("email", data.email()).await;
                match email {
                    Ok(result) => {
                        result.map_or(RegisterResult::Ok, |_| RegisterResult::Exists)
                    }
                    Err(_) => { RegisterResult::Other }
                }
            }
            Ok(Some(_)) => { RegisterResult::Exists }
            Err(_) => { RegisterResult::Other }
        }
    }

    async fn register(&self, mut data: LoginData) -> RegisterResult {
        let db = self.client.database("user").collection::<LoginData>("login");
        data.hash();
        let result = db.insert_one(data, None).await;

        match result {
            Ok(_) => { RegisterResult::Ok }
            Err(_) => { RegisterResult::Other }
        }
    }

    async fn validate_login(&self, data: &LoginData) -> LoginResult {
        let found = self.get_user("login", data.login()).await;

        match found {
            Ok(result) => {
                match result {
                    None => { LoginResult::NotExist }
                    Some(result) => {
                        let matches = bcrypt::verify(data.password(), result.password()).unwrap();
                        if matches {
                            LoginResult::Ok
                        } else {
                            LoginResult::IncorrectPassword
                        }
                    }
                }
            }
            Err(_) => { LoginResult::Other }
        }
    }
}