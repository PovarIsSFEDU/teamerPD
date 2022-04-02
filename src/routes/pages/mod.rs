use std::collections::HashMap;
use crate::prelude::*;
use rocket::fs::NamedFile;
use std::path::{PathBuf, Path};
use rocket::response::Redirect;
use rocket::State;
use crate::auth::Validator;
use rocket_dyn_templates::{Template};
use crate::auth::token::Token;
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

#[require_authorization(custom_handler, redirect_to = "/")]
#[get("/profile")]
pub async fn profile(token: Token, db: &State<MongoDriver>) -> Template {
    on_auth_failed! {
        let mut context = HashMap::new();
        context.insert("error", true);
        return Template::render("profile", context);
    }

    let login = token.claims.iss;
    let user = db.get_by_login::<User>(&login).await;
    println!("{:?}", user);
    match user {
        Ok(Some(res)) => {
            Template::render("profile", res)
        }
        _ => {
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
pub async fn teams(validator: Validator) -> Template {
    let mut context = HashMap::new();
    context.insert("auth", validator.validated);
    Template::render("teams", context)
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

#[get("/about")]
pub async fn about(validator: Validator) -> Template {
    let mut context = HashMap::new();
    context.insert("auth", validator.validated);
    Template::render("about", context)
}

#[get("/recover")]
pub async fn recover_password() -> Page {
    html_from_file(PATH, "templates/recover.html")
}

#[get("/emailverified")]
pub async fn email_verified() -> Page {
    html_from_file(PATH, "templates/emailverified.html")
}