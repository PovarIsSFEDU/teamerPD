mod prelude;
mod routes;
mod auth;

#[macro_use]
extern crate rocket;

use rocket::{Rocket, Build};
use routes::{pages, api};

#[launch]
fn launch() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![
            pages::main_page,
            pages::login,
            pages::team_by_id,
            pages::teams,
            pages::my_team,
            pages::admin_team,
            pages::files
        ])
        .mount("/api", routes![
            api::authenticate
        ])
}