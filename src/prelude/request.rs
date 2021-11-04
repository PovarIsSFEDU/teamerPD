use rocket::{Data, Request};
use rocket::data::ToByteUnit;

pub async fn body_to_string<'r>(req: &'r Request<'_>, data: Data<'r>) -> String {
    let length = req
        .headers()
        .get_one("Content-Length")
        .unwrap_or("2048")
        .parse::<i32>()
        .unwrap_or(2048)
        .bytes();

    data
        .open(length)
        .into_string()
        .await
        .unwrap()
        .value
}