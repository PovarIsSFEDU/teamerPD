mod prelude;
mod routes;
mod auth;
mod database;

#[macro_use]
extern crate rocket;

use rocket::{Rocket, Build};
use crate::routes::{pages, api};
use crate::database::MongoDriver;
use mongodb::options::ClientOptions;
use mongodb::Client;
use toml::Value;
use std::fs;

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
            pages::profile,
            pages::logout
        ])
        .mount("/api", routes![
            api::authenticate,
            api::register
        ])
        .mount("/", routes![pages::files])
}