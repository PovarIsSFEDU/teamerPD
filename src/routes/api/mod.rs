use std::path::Path;
use rocket::fs::TempFile;
use crate::auth::{token, LoginData, RegistrationData, Validator};
use crate::database::{DatabaseError, LoginResult, MongoDriver, RegistrationResult, TeamDataType, UserDataType, VerificationError, User, TeamCreationResult};
use crate::{crypto, mail, DOMAIN};
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::State;
use crate::auth::token::Token;
use crate::prelude;
use crate::teams::{TeamType};

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
            let mut link = DOMAIN.to_owned();
            link.push_str("/verify?key=");
            link.push_str(key.1.as_str());
            link.push_str("&user=");
            link.push_str(user.as_str());
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
            let mut key = user.login().clone();
            key.push_str(user.email().as_str());
            key.push_str(user.password().as_str());
            let key = crypto::hash_unique(key);

            let mut link = DOMAIN.to_owned();
            link.push_str("/recover?key=");
            link.push_str(key.as_str());
            link.push_str("&user=");
            link.push_str(user.login());

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
pub async fn upload_user(token: Token, u_type: &str, mut file: TempFile<'_>, db: &State<MongoDriver>) -> Status {
    let (name, ext) = generate_upload_name(&token.claims.iss, &file);
    let ext = ext.as_str();
    let (file_name, data_type, user);

    match u_type {
        "profile_photo" => {
            if let "jpg" | "jpeg" | "png" | "gif" = ext {
                file_name = format!("photo_{}", name);
                data_type = UserDataType::Photo;
                user = &token.claims.iss;
            } else {
                return Status::NotAcceptable;
            }
        }

        "resume" => {
            if let "doc" | "docx" | "pdf" = ext {
                file_name = format!("resume_{}", name);
                data_type = UserDataType::Resume;
                user = &token.claims.iss;
            } else {
                return Status::NotAcceptable;
            }
        }
        _ => return Status::BadRequest
    }

    let write_result = file
        .persist_to(
            Path::new("uploads/")
                .join(file_name.clone())
        )
        .await;
    if let Err(_) = write_result {
        return Status::InternalServerError;
    }

    let db_result = db
        .set_user_data(data_type, user, &file_name)
        .await;

    match db_result {
        Ok(_) => Status::Ok,
        Err(DatabaseError::NotFound) => Status::NotFound,
        Err(DatabaseError::Other) => Status::InternalServerError
    }
}

#[post("/upload_team?<u_type>", data = "<file>")]
pub async fn upload_team(token: Token, u_type: &str, mut file: TempFile<'_>, db: &State<MongoDriver>) -> Status {
    let (name, ext) = generate_upload_name(&token.claims.team.clone().unwrap(), &file);
    let ext = ext.as_str();
    let (file_name, data_type, team);

    match u_type {
        "logo" => {
            if let "jpg" | "jpeg" | "png" | "gif" = ext {
                file_name = format!("logo_{}", name);
                data_type = TeamDataType::Logo;
                team = token.claims.team.unwrap();
            } else {
                return Status::NotAcceptable
            }
        }

        _ => return Status::BadRequest
    }

    let write_result = file
        .persist_to(
            Path::new("uploads/")
                .join(file_name.clone())
        )
        .await;

    if let Err(_) = write_result {
        return Status::InternalServerError;
    }

    let db_result = db
        .set_team_data(data_type, &team, &file_name)
        .await;

    match db_result {
        Ok(_) => Status::Ok,
        Err(DatabaseError::NotFound) => Status::NotFound,
        Err(DatabaseError::Other) => Status::InternalServerError
    }
}

#[post("/create_team?<team_name>")]
pub async fn create_team(token: Token, team_name: String, db: &State<MongoDriver>) -> Status
{
    let captain = &token.claims.iss.clone();
    println!("{}",captain);
    let check = db.get_user_team(TeamType::Hackathon, captain).await;
    match check {
        None => {
            let res = db.create_team(TeamType::Conference, &team_name, captain ).await;
            match res {
                Ok(team) => {
                    println!("{} created!", team.name);
                    return  Status::Ok
                }
                Err(err) => {
                    match err {
                        TeamCreationResult::Ok => {return  Status::Ok}
                        TeamCreationResult::Exists => {return  Status::BadRequest}
                        TeamCreationResult::Other => {
                            println!("Error in creating team");
                            return Status::InternalServerError}
                    }
                }
            }
        }
        Some(str) => {return Status::BadRequest}
    }
    Status::NotImplemented
}

fn generate_upload_name(owner: &String, file: &TempFile<'_>) -> (String, String) {
    let name = file.name().unwrap().to_owned();
    let ext = prelude::get_ext(name);
    let name = crypto::hash(owner.as_bytes());
    (format!("{}.{}", name, ext), ext)
}

