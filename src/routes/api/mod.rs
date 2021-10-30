use crate::auth::LoginData;
use rocket::http::Status;
use rocket::State;
use crate::database::{DatabaseDriver, LoginResult, RegisterResult};
use rocket::response::status::Custom;

#[post("/auth", data = "<login_data>")]
pub async fn authenticate(login_data: LoginData, db: &State<Box<dyn DatabaseDriver>>) -> Custom<&str> {
    let validation_result = db.validate_login(&login_data).await;

    match validation_result {
        LoginResult::Ok => { Custom(Status::Ok, "") }
        LoginResult::IncorrectPassword => { Custom(Status::Forbidden, "Incorrect password") }
        LoginResult::NotExist => { Custom(Status::NotFound, "User with given login does not exist") }
        LoginResult::Other => { Custom(Status::InternalServerError, "Internal Server Error") }
    }
}

#[post("/register", data = "<login_data>", format = "application/json")]
pub async fn register(login_data: LoginData, db: &State<Box<dyn DatabaseDriver>>) -> Custom<&str> {
    let validation_result = db.validate_registration(&login_data).await;

    match validation_result {
        RegisterResult::Ok => {}
        RegisterResult::Exists => { return Custom(Status::BadRequest, "User with given login or email already exists") }
        RegisterResult::Other => { return Custom(Status::InternalServerError, "Internal Server Error") }
    }

    let registration_result = db.register(login_data).await;

    match registration_result {
        RegisterResult::Ok => { Custom(Status::Ok, "") }
        RegisterResult::Exists => { Custom(Status::BadRequest, "") }
        RegisterResult::Other => { Custom(Status::InternalServerError, "Internal Server Error") }
    }
}