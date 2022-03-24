pub mod user;
mod helpers;

use std::future::Future;
use crate::prelude::*;
use crate::routes::api::helpers::*;
use std::path::Path;
use crate::auth::{token, LoginData, RegistrationData, Validator};
use crate::database::{DatabaseError, LoginError, MongoDriver, RegistrationResult, TeamDataType, UserDataType, VerificationError, User, TeamCreationError, GetTeamError};
use crate::{crypto, mail, DOMAIN};
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::State;
use crate::auth::token::Token;
use crate::database::mongo::DatabaseOperationResult;
use crate::prelude;
use crate::teams::{TeamType};
use serde::{Serialize, Deserialize};

#[post("/auth", data = "<login_data>", format = "application/json")]
pub async fn authenticate(login_data: LoginData, db: &State<MongoDriver>) -> Custom<String> {
    let validation_result = db.validate_login(login_data).await;

    match validation_result {
        Ok(user) => {
            let token = token::issue(user);
            Custom(Status::Ok, token)
        }

        Err(LoginError::IncorrectPassword) =>
            Custom(Status::Forbidden, "Incorrect password".to_owned()),

        Err(LoginError::NotExist) =>
            Custom(Status::NotFound, "User with given login does not exist".to_owned()),

        Err(LoginError::Other) =>
            Custom(Status::InternalServerError, "Internal Server Error".to_owned())
    }
}

#[post("/register", data = "<registration_data>", format = "application/json")]
pub async fn register(registration_data: RegistrationData, db: &State<MongoDriver>) -> Custom<String> {
    let validation_result = db.validate_registration(&registration_data).await;

    match validation_result {
        RegistrationResult::Exists => {
            return Custom(Status::BadRequest, "User with given login or email already exists".to_owned());
        }

        RegistrationResult::Other => {
            return Custom(Status::InternalServerError, "Internal Server Error".to_owned());
        }

        RegistrationResult::Ok => {}
    }

    let registration_result = db.register(registration_data).await;

    match registration_result {
        Ok(user) => {
            let token = token::issue(user);
            Custom(Status::Created, token)
        }

        Err(_) => Custom(Status::InternalServerError, "Internal Server Error".to_owned()),
    }
}

#[get("/verify?<key>&<user>")]
pub async fn verify(key: String, user: String, db: &State<MongoDriver>) -> Custom<()> {
    let result = db.verify_email(key, user).await;

    match result {
        Ok(_) => Custom(Status::Ok, ()),
        Err(VerificationError::AlreadyVerified) => Custom(Status::Conflict, ()),
        Err(VerificationError::Other) => Custom(Status::InternalServerError, ()),
    }
}

#[get("/send_verification")]
pub async fn send_verification_link(db: &State<MongoDriver>, token: Token) -> Result<Status, Custom<&str>> {
    let user = token.claims.iss;

    let key = db.get_verification_key(user.clone()).await;
    match key {
        Ok(key) => {
            let link = DOMAIN
                .concat("/verify?key=")
                .concat(key.1)
                .concat("&user=")
                .concat(user)
                .into_string();

            let result = mail::send_email_verification(key.0, link);

            match result {
                Ok(_) => Ok(Status::Ok),
                Err(_) => Err(Custom(Status::InternalServerError, "Something went wrong while sending the email"))
            }
        }

        Err(DatabaseError::NotFound) => Err(Custom(Status::NotFound, "User or key not found")),

        Err(DatabaseError::Other) => {
            Err(Custom(Status::InternalServerError, "Internal Server Error"))
        }
    }
}

#[post("/recover?<key>", data = "<data>")]
pub async fn recover_password(key: String, data: LoginData, db: &State<MongoDriver>) -> Custom<()> {
    let result = db.validate_recovery(key, data).await;

    result.into_custom()
}

#[get("/send_recovery?<user>")]
pub async fn send_password_recovery(user: String, db: &State<MongoDriver>) -> Result<Status, Custom<&str>> {
    let user = db.get::<RegistrationData>("login", &user).await;

    match user {
        Ok(None) => Err(Custom(Status::NotFound, "User with given login not found")),
        Err(_) => Err(Custom(Status::InternalServerError, "Internal Server Error")),

        Ok(Some(user)) => {
            let key = user
                .login()
                .concat(user.email())
                .concat(user.password())
                .into_string();
            let key = crypto::hash_unique(key);

            let link = DOMAIN
                .concat("/recover?key=")
                .concat(key.as_str())
                .concat("&user=")
                .concat(user.login())
                .into_string();

            let result = db.set_recovery_key(&user, &key).await;

            match result {
                Ok(_) => {
                    let result = mail::send_recovery(user.email().clone(), link);
                    match result {
                        Ok(_) => Ok(Status::Ok),
                        Err(_) => Err(Custom(Status::InternalServerError, "Something went wrong while sending the email"))
                    }
                }

                Err(DatabaseError::NotFound) =>
                    Err(Custom(Status::NotFound, "Could not set key: user not found")),
                Err(DatabaseError::Other) =>
                    Err(Custom(Status::InternalServerError, "Could not set key: Internal error")),
            }
        }
    }
}

#[post("/update_user", data="<user>")]
pub async fn update_user(_token: Token, user: User, db: &State<MongoDriver>) -> Status {
    let mut result = Status::Ok;

    result = match db.update_user(user.clone()).await {
        Ok(_) => Status::Ok,
        Err(_) => Status::InternalServerError
    };
    return result;
}

#[post("/upload?<u_type>", data = "<file>")]
pub async fn upload(token: Token, u_type: &str, file: TempFile<'_>, db: &State<MongoDriver>) -> Custom<()> {
    let data = match u_type {
        u_type @ ("profile_photo" | "resume") => generate_user_data(&token, u_type, &file),

        #[allow(unused_parens)]
        u_type @ ("logo") => generate_team_data(&token, u_type, &file),

        _ => return Custom(Status::BadRequest, ())
    };

    let (file_name, data_type) = match data {
        Ok(data) => data,
        Err(status) => return Custom(status, ())
    };

    let save_result = save_upload(file, file_name.clone()).await;
    if save_result.is_err() {
        return Custom(Status::InternalServerError, ());
    }

    let db_result = match data_type {
        User(t) =>
            db
                .set_user_data(t, &token.claims.iss, &file_name)
                .await,

        Team(t) =>
            db
                .set_team_data(t, &token.claims.team.unwrap(), &file_name)
                .await
    };

    db_result.into_custom()
}

#[post("/create_team?<team_name>")]
pub async fn create_team(token: Token, team_name: String, db: &State<MongoDriver>) -> Status {
    let captain = token.claims.iss;
    println!("{}", captain);
    let check = db.get_user_team(TeamType::Hackathon, &captain).await;
    match check {
        Err(GetTeamError::NotInTeam) => {
            let creation_result = db.create_team(TeamType::Hackathon, team_name, captain).await;

            match creation_result {
                Ok(team) => {
                    println!("{} created!", team.name);
                    let result = db.set_user_data(UserDataType::TeamName, &team.captain, &team.name).await;

                    match result {
                        Ok(_) => Status::Ok,
                        Err(_) => Status::InternalServerError
                    }
                }

                Err(TeamCreationError::Other) => {
                    println!("Error creating team");
                    Status::InternalServerError
                }
                Err(TeamCreationError::Exists) => Status::BadRequest
            }
        }

        Err(GetTeamError::NotFound) => Status::BadRequest,
        Err(GetTeamError::Other) => Status::InternalServerError,
        Ok(_) => Status::BadRequest
    }
}

#[get("/get_teams")]
pub async fn get_teams(db: &State<MongoDriver>) -> Result<String, Status> {
    match db.get_teams().await {
        Err(_) => Err(Status::InternalServerError),
        Ok(teams) => Ok(serde_json::to_string(&teams).unwrap())
    }
}

fn generate_user_data(token: &Token, u_type: &str, file: &TempFile<'_>) -> Result<(String, UploadDataType), Status> {
    let name = generate_upload_name(&token.claims.iss, file);

    match u_type {
        "profile_photo" => match is_image(&name) {
            true => Ok((format!("photo_{}", name), User(UserDataType::Photo))),
            false => Err(Status::NotAcceptable),
        }

        "resume" => match is_doc(&name) {
            true => Ok((format!("resume_{}", name), User(UserDataType::Resume))),
            false => Err(Status::NotAcceptable),
        }

        _ => Err(Status::BadRequest)
    }
}

fn generate_team_data(token: &Token, u_type: &str, file: &TempFile<'_>) -> Result<(String, UploadDataType), Status> {
    if token.claims.team.is_none() {
        return Err(Status::Forbidden);
    }

    let name = generate_upload_name(token.claims.team.as_ref().unwrap(), file);

    match u_type {
        "logo" => match is_image(&name) {
            true => Ok((format!("logo_{}", name), Team(TeamDataType::Logo))),
            false => Err(Status::NotAcceptable),
        }

        _ => Err(Status::BadRequest)
    }
}