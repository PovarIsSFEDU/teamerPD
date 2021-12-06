mod prelude;
mod routes;
mod auth;
mod database;
mod mail;
mod crypto;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate teamer_proc_macro;
#[macro_use]
extern crate rocket_dyn_templates;

use rocket::{Rocket, Build};
use crate::routes::{pages, api};
use crate::database::MongoDriver;
use mongodb::options::ClientOptions;
use mongodb::Client;
use crate::prelude::*;
use toml::Value;
use std::fs;
use rocket_dyn_templates::Template;

#[cfg(debug_assertions)]
pub const DOMAIN: &str = "http://127.0.0.1:8000";

//Change when actual domain is known
#[cfg(not(debug_assertions))]
pub const DOMAIN: &str = "http://teamer.firon.org";

#[launch]
async fn launch() -> Rocket<Build> {
    let db_link = from_config("db_link");

    let db = ClientOptions::parse(db_link)
        .await
        .unwrap();
    let client = Client::with_options(db).unwrap();
    let client = MongoDriver::new(client);
    rocket::build()
        .manage(client)
        .mount("/api", routes![
            api::authenticate,
            api::register,
            api::send_verification_link,
            api::send_password_recovery,
            api::upload
        ])
        .mount("/", routes![pages::files])
        .mount("/", routes![
            pages::main_page,
            pages::login,
            pages::team_by_id,
            pages::teams,
            pages::my_team,
            pages::admin_team,
            pages::recover_password,
            pages::email_verified,
            api::verify,
            api::recover_password,
            pages::profile,
            pages::logout
        ])
        .attach(Template::fairing())
}