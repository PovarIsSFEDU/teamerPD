use std::collections::HashMap;
use crate::prelude::*;
use rocket::fs::NamedFile;
use std::path::{PathBuf, Path};
use rocket::response::Redirect;
use rocket::{Request, State};
use crate::auth::Validator;
use rocket_dyn_templates::{Template};
use crate::database::{MongoDriver, User};

const PATH: &str = "resources/";

#[get("/<file..>", rank = 2)]
pub async fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new(PATH).join(file)).await.ok()
}

#[get("/")]
pub async fn main_page(validator: Validator) -> Template {
    let mut context = HashMap::new();
    context.insert("auth", validator.validated);
    Template::render("main", context)
}

#[get("/login")]
pub async fn login(validator: Validator) -> Result<Redirect, Page> {
    match validator.validated {
        true => Ok(Redirect::to(uri!("/"))),
        false => Err(html_from_file(PATH, "templates/login.html"))
    }
}

#[get("/logout")]
pub async fn logout() -> Redirect {
    Redirect::to(uri!("/login"))
}

// #[require_authorization]
#[get("/profile/<login>")]
pub async fn profile(login: String, db: &State<MongoDriver>, validator: Validator) -> Template {
    match validator.validated {
        true => {
            let user = db.get_by_name::<User>(&login).await;
            println!("{:?}", user);
            match user {
                Ok(Some(res)) => {
                    println!("{:?}", res);
                    Template::render("profile", res)
                }
                _ => {
                    let mut context = HashMap::new();
                    context.insert("error", true);
                    Template::render("profile", context)
                }
            }
        }
        false => {
            let mut context = HashMap::new();
            context.insert("error", true);
            Template::render("profile", context)
        }
    }
}

#[require_authorization(redirect_to = "/login", custom, cus)]
#[get("/team/<id>")]
pub async fn team_by_id(id: i32) -> Result<Page, Redirect> {
    Ok(html_from_file(PATH, "templates/team.html"))
}


#[get("/teams")]
pub async fn teams() -> Page {
    html_from_file(PATH, "templates/teams.html")
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