use crate::prelude::*;
use rocket::fs::NamedFile;
use std::path::{PathBuf, Path};

const PATH: &str = "resources/";

#[get("/<file..>")]
pub async fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new(PATH).join(file)).await.ok()
}

#[get("/")]
pub async fn main_page() -> Page {
    html_from_file(PATH, "templates/main.html")
}

#[get("/login")]
pub async fn login() -> Page {
    html_from_file(PATH, "templates/login.html")
}

#[get("/team/<id>")]
pub async fn team_by_id(id: i32) -> Page {
    html_from_file(PATH, "templates/team.html")
}

#[get("/teams")]
pub async fn teams() -> Page {
    html_from_file(PATH, "templates/teams.html")
}

#[get("/myteam")]
pub async fn my_team() -> Page {
    html_from_file(PATH, "templates/team.html")
}

#[get("/admteam")]
pub async fn admin_team() -> Page {
    html_from_file(PATH, "templates/team.html")
}