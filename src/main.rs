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

use rocket::{Rocket, Build};
use crate::routes::{pages, api};
use crate::database::MongoDriver;
use mongodb::options::ClientOptions;
use mongodb::Client;
use toml::Value;
use std::fs;

#[cfg(debug_assertions)]
pub const DOMAIN: &str = "http://127.0.0.1:8000";

//Change when actual domain is known
#[cfg(not(debug_assertions))]
pub const DOMAIN: &str = "http://127.0.0.1:8000";

#[launch]
async fn launch() -> Rocket<Build> {
    let toml = fs::read_to_string("Config.toml").expect("Could not open toml");
    let value = toml.as_str().parse::<Value>().unwrap();
    let db_link = value["db_link"].as_str().unwrap();

    let db = ClientOptions::parse(db_link)
        .await
        .unwrap();
    let client = Client::with_options(db).unwrap();
    let client = MongoDriver::new(client);
    rocket::build()
        .manage(client)
        .mount("/", routes![
            pages::main_page,
            pages::login,
            pages::team_by_id,
            pages::teams,
            pages::my_team,
            pages::admin_team,
            api::verify,
            api::recover_password
        ])
        .mount("/api", routes![
            api::authenticate,
            api::register,
            api::send_verification_link,
            api::send_password_recovery
        ])
        .mount("/", routes![pages::files])
}