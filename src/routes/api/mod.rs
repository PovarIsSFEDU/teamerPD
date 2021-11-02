use crate::auth::{RegistrationData, LoginData, token};
use crate::database::{LoginResult, MongoDriver, RegisterResult};
use rocket::http::Status;
use rocket::State;
use rocket::response::status::Custom;

#[post("/auth", data = "<login_data>", format = "application/json")]
pub async fn authenticate(login_data: LoginData, db: &State<MongoDriver>) -> Custom<String> {
    let validation_result = db.validate_login(&login_data).await;

    match validation_result {
        Ok(user) => {
            let token = token::issue(user);
            let token = serde_json::to_string(&token).unwrap();
            Custom(Status::Ok, token)
        }

        Err(LoginResult::IncorrectPassword) =>
            Custom(Status::Forbidden, "Incorrect password".to_owned()),

        Err(LoginResult::NotExist) =>
            Custom(Status::NotFound, "User with given login does not exist".to_owned()),

        Err(LoginResult::Other) =>
            Custom(Status::InternalServerError, "Internal Server Error".to_owned()),
    }
}

#[post("/register", data = "<login_data>", format = "application/json")]
pub async fn register(login_data: RegistrationData, db: &State<MongoDriver>) -> Custom<String> {
    let validation_result = db.validate_registration(&login_data).await;

    match validation_result {
        RegisterResult::Ok => {}

        RegisterResult::Exists =>
            return Custom(Status::BadRequest, "User with given login or email already exists".to_owned()),

        RegisterResult::Other =>
            return Custom(Status::InternalServerError, "Internal Server Error".to_owned())
    }

    let registration_result = db.register(login_data).await;

    match registration_result {
        Ok(user) => {
            let token = token::issue(user);
            let token = serde_json::to_string(&token).unwrap();
            Custom(Status::Ok, token)
        }

        Err(_) => Custom(Status::InternalServerError, "Internal Server Error".to_owned())
    }
}