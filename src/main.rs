mod prelude;

#[macro_use]
extern crate rocket;

use rocket::fs::NamedFile;
use rocket::{Rocket, Build};
use prelude::*;

//TODO Find out actual path to working directory and change
const PATH_TO_HTML: &str = "resources/";

#[launch]
fn launch() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![main_page, login, team_by_id, teams, my_team, admin_team, script])
}

//Dummy for scripts and styles to work properly
//TODO Rewrite to use dedicated directory for resources
#[get("/<script>")]
async fn script(script: String) -> Option<NamedFile> {
    NamedFile::open(concat(PATH_TO_HTML, script.as_str())).await.ok()
}

#[get("/")]
async fn main_page() -> Page {
    html_from_file(PATH_TO_HTML, "index.html")
}

#[get("/login")]
async fn login() -> Page {
    html_from_file(PATH_TO_HTML, "login.html")
}

#[get("/team/<id>")]
async fn team_by_id(id: i32) -> Page {
    html_from_file(PATH_TO_HTML, "team.html")
}

#[get("/teams")]
async fn teams() -> Page {
    html_from_file(PATH_TO_HTML, "teams.html")
}

#[get("/myteam")]
async fn my_team() -> Page {
    html_from_file(PATH_TO_HTML, "team.html")
}

#[get("/admteam")]
async fn admin_team() -> Page {
    html_from_file(PATH_TO_HTML, "team.html")
}