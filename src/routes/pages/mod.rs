use crate::prelude::*;
use rocket::fs::NamedFile;
use std::path::{PathBuf, Path};
use rocket::response::Redirect;
use crate::auth::Validator;

const PATH: &str = "resources/";

#[get("/<file..>")]
pub async fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new(PATH).join(file)).await.ok()
}

#[get("/")]
pub async fn main_page() -> Page { html_from_file(PATH, "templates/main.html") }

#[get("/login")]
pub async fn login(validator: Validator) -> Result<Page, Page> {
    match validator.validated {
        true => Ok(html_from_file(PATH, "templates/main.html")),
        false => Err(html_from_file(PATH, "templates/login.html"))
    }
}

#[get("/profile")]
pub async fn profile(validator: Validator) -> Result<Page, Redirect> {
    match validator.validated {
        true => Ok(html_from_file(PATH, "templates/teams.html")),
        false => Err(Redirect::to(uri!("/login")))
    }
}

#[get("/team/<id>")]
pub async fn team_by_id(validator: Validator, id: i32) -> Result<Page, Redirect> {
    match validator.validated {
        true => Ok(html_from_file(PATH, "templates/team.html")),
        false => Err(Redirect::to(uri!("/login")))
    }
}

#[get("/teams")]
pub async fn teams(validator: Validator) -> Result<Page, Redirect> {
    match validator.validated {
        true => Ok(html_from_file(PATH, "templates/teams.html")),
        false => Err(Redirect::to(uri!("/login")))
    }
}

#[get("/myteam")]
pub async fn my_team(validator: Validator) -> Result<Page, Redirect> {
    match validator.validated {
        true => Ok(html_from_file(PATH, "templates/team.html")),
        false => Err(Redirect::to(uri!("/login")))
    }
}

#[get("/admteam")]
pub async fn admin_team(validator: Validator) -> Result<Page, Redirect> {
    match validator.validated {
        true => Ok(html_from_file(PATH, "templates/team.html")),
        false => Err(Redirect::to(uri!("/login")))
    }
}