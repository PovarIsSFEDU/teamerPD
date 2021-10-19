use crate::auth::{self, LoginData};
use rocket::http::Status;

#[post("/auth", data = "<login_data>")]
pub async fn authenticate(login_data: LoginData) -> Status {
    auth::check_login(&login_data);
    Status::Ok
}