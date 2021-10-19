use crate::prelude::*;
use rocket::fs::NamedFile;

const PATH: &str = "resources/";

//Dummy for scripts and styles to work properly
//TODO Rewrite to use dedicated directory for resources
#[get("/<script>")]
pub async fn script(script: String) -> Option<NamedFile> {
    NamedFile::open(concat(PATH, script.as_str())).await.ok()
}

#[get("/")]
pub async fn main_page() -> Page {
    html_from_file(PATH, "index.html")
}

#[get("/login")]
pub async fn login() -> Page {
    html_from_file(PATH, "login.html")
}

#[get("/team/<id>")]
pub async fn team_by_id(id: i32) -> Page {
    html_from_file(PATH, "team.html")
}

#[get("/teams")]
pub async fn teams() -> Page {
    html_from_file(PATH, "teams.html")
}

#[get("/myteam")]
pub async fn my_team() -> Page {
    html_from_file(PATH, "team.html")
}

#[get("/admteam")]
pub async fn admin_team() -> Page {
    html_from_file(PATH, "team.html")
}