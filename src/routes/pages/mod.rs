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
use serde::{Serialize, Deserialize};
use crate::database::task::Task;
use crate::database::team::Team;
use crate::teams::TeamType;

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
#[get("/team/<name>")]
pub async fn team_by_id(name: String) -> Result<Page, Redirect> {
    Ok(html_from_file(PATH, "templates/team.html"))
}

#[require_authorization(redirect_to = "/login", custom, cus)]
#[get("/my_teams")]
pub async fn my_teams(token: Token, db: &State<MongoDriver>) -> Template {
    #[derive(Serialize, Deserialize)]
    struct Insertion {
        pub auth: bool,
        pub team: Option<Team>,
        pub tasks: Option<Vec<Task>>,
    }
    let user = token.claims.iss;
    let t = db.get_user_team(TeamType::Hackathon, &user).await.unwrap();
    let team = db
        .get_team::<Team>("name", &t)
        .await
        .unwrap()
        .unwrap();
    Template::render("my_teams", Insertion { auth: validator.validated, team: Some(team), tasks: Some(db.get_tasks(&t).await.unwrap()) })
}


#[get("/teams")]
pub async fn teams(validator: Validator) -> Template {
    let mut context = HashMap::new();
    context.insert("auth", validator.validated);
    Template::render("teams", context)
}

#[require_authorization(redirect_to = "/login", custom, cus)]
#[get("/users")]
pub async fn users() -> Template {
    let mut context = HashMap::new();
    context.insert("auth", validator.validated);
    Template::render("users", context)
}

#[require_authorization(redirect_to = "/login", custom, cus)]
#[get("/<login>")]
pub async fn get_one_user(login: String, db: &State<MongoDriver>) -> Template {
    #[derive(Serialize, Deserialize)]
    struct Insertion {
        pub auth: bool,
        pub user: Option<User>,
    }
    let user = db.get_by_login::<User>(&login).await;
    match user {
        Ok(Some(res)) => {
            Template::render("one_user", Insertion { auth: validator.validated, user: Some(res) })
        }
        _ => {
            Template::render("one_user", Insertion { auth: validator.validated, user: None })
        }
    }
}

#[require_authorization(redirect_to = "/login", custom, cus)]
#[get("/create_team")]
pub async fn create_team() -> Template {
    let mut context = HashMap::new();
    context.insert("auth", validator.validated);
    Template::render("create_team", context)
}

#[require_authorization(redirect_to = "/login", custom, cus)]
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