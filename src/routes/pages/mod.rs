use crate::prelude::*;
use rocket::fs::NamedFile;
use std::path::{PathBuf, Path};
use rocket::response::Redirect;
use crate::auth::Validator;

const PATH: &str = "resources/";

#[get("/<file..>", rank = 2)]
pub async fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new(PATH).join(file)).await.ok()
}

#[get("/")]
pub async fn main_page() -> Page { html_from_file(PATH, "templates/main.html") }

#[get("/login")]
pub async fn login() -> Page {
    html_from_file(PATH, "templates/login.html")
}

#[require_authorization(redirect_to = "/login", custom, cus)]
#[get("/team/<id>")]
pub async fn team_by_id(id: i32) -> Result<Page, Redirect> {
    Ok(html_from_file(PATH, "templates/team.html"))
}

#[require_authorization]
#[get("/teams")]
pub async fn teams() -> Result<Page, Redirect> {
    Ok(html_from_file(PATH, "templates/teams.html"))
}

#[require_authorization]
#[get("/myteam")]
pub async fn my_team() -> Result<Page, Redirect> {
    Ok(html_from_file(PATH, "templates/team.html"))
}

#[require_authorization]
#[get("/admteam")]
pub async fn admin_team() -> Result<Page, Redirect> {
    Ok(html_from_file(PATH, "templates/team.html"))
}

#[get("/recover")]
pub async fn recover_password() -> Page {
    html_from_file(PATH, "templates/recover.html")
}