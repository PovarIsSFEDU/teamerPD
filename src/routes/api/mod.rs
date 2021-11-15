use std::path::Path;
use rocket::fs::TempFile;
use crate::auth::{token, LoginData, RegistrationData, Validator};
use crate::database::{DatabaseError, LoginResult, MongoDriver, RegistrationResult, TeamDataType, UserDataType, VerificationError};
use crate::{crypto, mail, DOMAIN};
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::State;
use crate::auth::token::Token;
use crate::prelude::*;
use crate::prelude::concat::Concatenate;

#[post("/auth", data = "<login_data>", format = "application/json")]
pub async fn authenticate(login_data: LoginData, db: &State<MongoDriver>) -> Custom<String> {
    let validation_result = db.validate_login(login_data).await;

    match validation_result {
        Ok(user) => {
            let token = token::issue(user);
            Custom(Status::Ok, token)
        }

        Err(LoginResult::IncorrectPassword) =>
            Custom(Status::Forbidden, "Incorrect password".to_owned()),

        Err(LoginResult::NotExist) =>
            Custom(Status::NotFound, "User with given login does not exist".to_owned()),

        Err(LoginResult::Other) =>
            Custom(Status::InternalServerError, "Internal Server Error".to_owned())
    }
}

#[post("/register", data = "<registration_data>", format = "application/json")]
pub async fn register(registration_data: RegistrationData, db: &State<MongoDriver>) -> Custom<String> {
    let validation_result = db.validate_registration(&registration_data).await;

    match validation_result {
        RegistrationResult::Exists =>
            return Custom(Status::BadRequest, "User with given login or email already exists".to_owned()),

        RegistrationResult::Other =>
            return Custom(Status::InternalServerError, "Internal Server Error".to_owned()),

        RegistrationResult::Ok => {}
    }

    let registration_result = db.register(registration_data).await;

    match registration_result {
        Ok(user) => {
            let token = token::issue(user);
            Custom(Status::Created, token)
        }

        Err(_) => Custom(Status::InternalServerError, "Internal Server Error".to_owned())
    }
}

#[get("/verify?<key>&<user>")]
pub async fn verify(key: String, user: String, db: &State<MongoDriver>) -> Status {
    let result = db.verify_email(key, user).await;

    match result {
        Ok(_) => Status::Ok,
        Err(VerificationError::AlreadyVerified) => Status::Conflict,
        Err(VerificationError::Other) => Status::InternalServerError,
    }
}

#[require_authorization(custom_handler)]
#[get("/send_verification?<user>")]
pub async fn send_verification_link(user: String, db: &State<MongoDriver>) -> Result<Status, Custom<&str>> {
    on_auth_failed! {
        return Err(Custom(Status::Forbidden, "Not authorized"));
    }

    let key = db.get_verification_key(user.clone()).await;
    match key {
        Ok(key) => {
            let link = DOMAIN
                .concat("/verify?key=")
                .concat(key.1)
                .concat("&user=")
                .concat(user)
                .to_string();
            //Uncomment when SMTP is working
            //mail::send_email_verification(key.0, link);
            Ok(Status::Ok)
        }

        Err(DatabaseError::NotFound) => Err(Custom(Status::NotFound, "User or key not found")),

        Err(DatabaseError::Other) => {
            Err(Custom(Status::InternalServerError, "Internal Server Error"))
        }
    }
}

#[post("/recover?<key>", data = "<data>")]
pub async fn recover_password(key: String, data: LoginData, db: &State<MongoDriver>) -> Status {
    let result = db.validate_recovery(key, data).await;

    match result {
        Ok(_) => Status::Ok,
        Err(DatabaseError::NotFound) => Status::NotFound,
        Err(DatabaseError::Other) => Status::InternalServerError,
    }
}

#[get("/send_recovery?<user>")]
pub async fn send_password_recovery(user: String, db: &State<MongoDriver>) -> Result<Status, Custom<&str>> {
    let user = db.get::<RegistrationData>("login", &user).await;

    match user {
        Ok(None) => Err(Custom(Status::NotFound, "User with given login not found")),
        Err(_) => Err(Custom(Status::InternalServerError, "Internal Server Error")),

        Ok(Some(user)) => {
            let key = user.login()
                .concat(user.email())
                .concat(user.password())
                .to_string();
            let key = crypto::hash_unique(key);

            let link = DOMAIN
                .concat("/recover?key=")
                .concat(key.as_str())
                .concat("&user=")
                .concat(user.login())
                .to_string();

            let result = db.set_recovery_key(&user, &key).await;

            match result {
                Ok(_) => {
                    //mail::send_recovery(user.email().clone(), link);
                    Ok(Status::Ok)
                }

                Err(DatabaseError::NotFound) => Err(Custom(Status::NotFound, "Could not set key: user not found")),
                Err(DatabaseError::Other) => Err(Custom(Status::InternalServerError, "Could not set key: Internal error"))
            }
        }
    }
}

#[post("/upload_user?<u_type>", data = "<file>")]
pub async fn upload_user(token: Token, u_type: &str, file: TempFile<'_>, db: &State<MongoDriver>) -> Custom<()> {
    let name = generate_upload_name(&token.claims.iss, &file);
    let (file_name, data_type);

    match u_type {
        "profile_photo" => {
            if !is_image(&name) {
                return Custom(Status::NotAcceptable, ());
            }

            file_name = format!("photo_{}", name);
            data_type = UserDataType::Photo;
        }

        "resume" => {
            if !is_doc(&name) {
                return Custom(Status::NotAcceptable, ());
            }

            file_name = format!("resume_{}", name);
            data_type = UserDataType::Resume;
        }

        _ => return Custom(Status::BadRequest, ())
    }

    let write_result = save_upload(file, file_name.clone()).await;

    if let Err(_) = write_result {
        return Custom(Status::InternalServerError, ());
    }

    let user = &token.claims.iss;
    let db_result = db
        .set_user_data(data_type, user, &file_name)
        .await;

    match db_result {
        Ok(_) => Custom(Status::Ok, ()),
        Err(DatabaseError::NotFound) => Custom(Status::NotFound, ()),
        Err(DatabaseError::Other) => Custom(Status::InternalServerError, ())
    }
}

#[post("/upload_team?<u_type>", data = "<file>")]
pub async fn upload_team(token: Token, u_type: &str, file: TempFile<'_>, db: &State<MongoDriver>) -> Custom<()> {
    if token.claims.team.is_none() {
        return Custom(Status::Forbidden, ())
    }

    let name = generate_upload_name(&token.claims.team.clone().unwrap(), &file);
    let (file_name, data_type);

    match u_type {
        "logo" => {
            if !is_image(&name) {
                return Custom(Status::NotAcceptable, ())
            }

            file_name = format!("logo_{}", name);
            data_type = TeamDataType::Logo;
        }

        _ => return Custom(Status::BadRequest, ())
    }

    let write_result = save_upload(file, file_name.clone()).await;

    if let Err(_) = write_result {
        return Custom(Status::InternalServerError, ());
    }

    let team = token.claims.team.unwrap();
    let db_result = db
        .set_team_data(data_type, &team, &file_name)
        .await;

    match db_result {
        Ok(_) => Custom(Status::Ok, ()),
        Err(DatabaseError::NotFound) => Custom(Status::NotFound, ()),
        Err(DatabaseError::Other) => Custom(Status::InternalServerError, ())
    }
}

fn generate_upload_name(owner: &String, file: &TempFile<'_>) -> String {
    let name = file.name().unwrap().to_owned();
    let ext = get_ext(&name);
    let name = crypto::hash(owner.as_bytes());
    format!("{}.{}", name, ext)
}

async fn save_upload(mut file: TempFile<'_>, name: String) -> std::io::Result<()> {
    file
        .persist_to(Path::new("uploads/").join(name))
        .await
}
