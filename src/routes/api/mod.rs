use crate::auth::{self, LoginData};
use rocket::http::Status;

#[post("/auth", data = "<login_data>")]
pub async fn authenticate(login_data: LoginData) -> Status {
    auth::validate_login(&login_data);
    Status::Ok
}