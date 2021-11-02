mod prelude;
mod routes;
mod auth;
mod database;

#[macro_use]
extern crate rocket;

use rocket::{Rocket, Build};
use crate::routes::{pages, api};
use crate::database::{DatabaseDriver, MongoDriver};
use mongodb::options::ClientOptions;
use mongodb::Client;

#[launch]
async fn launch() -> Rocket<Build> {
    let db = ClientOptions::parse("mongodb+srv://pd:D0SEoobFKGWLM98R@cluster0.us09s.mongodb.net/myFirstDatabase?retryWrites=true&w=majority")
        .await
        .unwrap();
    let client = Client::with_options(db).unwrap();
    let client = MongoDriver::new(client);
    rocket::build()
        .manage(Box::new(client) as Box<dyn DatabaseDriver>)
        .mount("/", routes![
            pages::main_page,
            pages::login,
            pages::team_by_id,
            pages::teams,
            pages::my_team,
            pages::admin_team
        ])
        .mount("/api", routes![
            api::authenticate,
            api::register
        ])
        .mount("/", routes![pages::files])
}