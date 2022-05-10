pub mod user;
mod helpers;

use std::time::{SystemTime, UNIX_EPOCH};
use crate::prelude::*;
use crate::routes::api::helpers::*;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use crate::auth::{token, LoginData, RegistrationData};
use crate::database::{DatabaseError, LoginError, MongoDriver, RegistrationResult, TeamDataType, UserDataType, VerificationError, User, team::Team, TeamCreationError, GetTeamError, task::Task};
use crate::{crypto, mail, DOMAIN};
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::State;
use crate::auth::token::Token;
use crate::teams::{TeamType};
use serde::{Serialize, Deserialize};
use crate::database::AddUserToTeamResult;


#[derive(Serialize, Deserialize)]
struct InvitationData {
    exp: u128,
    pub team: String,
    pub usr: String,
}

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

#[post("/update_user", data = "<user>")]
pub async fn update_user(_token: Token, user: User, db: &State<MongoDriver>) -> Status {
    match db.update_user(user.clone()).await {
        Ok(_) => Status::Ok,
        Err(_) => Status::InternalServerError
    }
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

#[post("/create_team", data = "<team>")]
pub async fn create_team(token: Token, team: Team, db: &State<MongoDriver>) -> Status {
    let captain = token.claims.iss;
    let check = db.get_user_team(TeamType::Hackathon, &captain).await;
    match check {
        Err(GetTeamError::NotInTeam) => {
            let creation_result = db.create_team(TeamType::Hackathon, team, captain).await;

            match creation_result {
                Ok(team) => {
                    let result = db.set_user_data(UserDataType::TeamName, &team.captain, &team.name).await;

                    match result {
                        Ok(_) => Status::Ok,
                        Err(_) => Status::InternalServerError
                    }
                }

                Err(TeamCreationError::Other) => {
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

#[post("/add_to_team?<user>&<team>")]
pub async fn add_to_team(token: Token, user: String, team: String, db: &State<MongoDriver>) -> Status
{
    match db.check_is_captain(&team, &token.claims.iss).await
    {
        Ok(true) => {}
        Ok(false) => { return Status::Forbidden; }
        Err(..) => return Status::InternalServerError
    }
    match db.add_user_to_team(&team, &user).await
    {
        AddUserToTeamResult::Ok => Status::Ok,
        AddUserToTeamResult::UserNotFound => Status::NotFound,
        AddUserToTeamResult::TeamNotFound => Status::NotFound,
        AddUserToTeamResult::Exists => Status::BadRequest,
        AddUserToTeamResult::Error => Status::InternalServerError,
        AddUserToTeamResult::Other => Status::InternalServerError
    }
}

#[get("/get_all_teams?<page>")]
pub async fn get_teams(db: &State<MongoDriver>, mut page: usize) -> Result<String, Status> {
    if page < 1 {
        page = 1
    }
    match db.get_teams(page, 6).await {
        Err(_) => Err(Status::InternalServerError),
        Ok(teams) => Ok(serde_json::to_string(&teams).unwrap())
    }
}

#[get("/get_teams_pagination")]
pub async fn get_teams_pagination(db: &State<MongoDriver>) -> Result<String, Status> {
    match db.get_teams_pages().await {
        Err(_) => Err(Status::InternalServerError),
        Ok(pages_count) => Ok(serde_json::to_string(&pages_count).unwrap())
    }
}

#[get("/get_all_users?<page>")]
pub async fn get_users(db: &State<MongoDriver>, page: usize) -> Result<String, Status> {
    if page < 1 {
        return match db.get_users(0, 10000).await {
            Err(_) => Err(Status::InternalServerError),
            Ok(users) => Ok(serde_json::to_string(&users).unwrap())
        };
    }
    match db.get_users(page, 5).await {
        Err(_) => Err(Status::InternalServerError),
        Ok(users) => Ok(serde_json::to_string(&users).unwrap())
    }
}

#[get("/get_users_pagination")]
pub async fn get_users_pagination(db: &State<MongoDriver>) -> Result<String, Status> {
    match db.get_users_pages().await {
        Err(_) => Err(Status::InternalServerError),
        Ok(pages_count) => Ok(serde_json::to_string(&pages_count).unwrap())
    }
}

#[get("/find_user?<username>")]
pub async fn find_user(db: &State<MongoDriver>, username: String) -> Result<String, Status> {
    match db.find_by_username(username).await {
        Err(_) => Err(Status::InternalServerError),
        Ok(users) => Ok(serde_json::to_string(&users).unwrap())
    }
}

#[get("/send_invitation?<user>")]
pub async fn send_invitation(token: Token, db: &State<MongoDriver>, user: String) -> Status {
    let sender = token.claims.iss;
    let sender_team = db.get_user_team(TeamType::Hackathon, &sender).await;
    let sender_team = match sender_team {
        Ok(team) => team,
        Err(GetTeamError::NotInTeam) => return Status::Forbidden,
        _ => return Status::InternalServerError
    };


    match db.get_user_team(TeamType::Hackathon, &user).await {
        Err(GetTeamError::NotFound | GetTeamError::Other) => Status::InternalServerError,
        _ => {
            let email_header = format!("You are invited to join {}", sender_team.clone());
            let data = InvitationData {
                team: sender_team.clone(),
                usr: user.clone(),
                exp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() + 2 * 24 * 60 * 60 * 1000,
            };
            let key = jsonwebtoken::encode(
                &Header::default(),
                &data,
                &EncodingKey::from_secret(from_config("jwt_secret").as_bytes()),
            ).unwrap();

            let link = DOMAIN
                .concat("/join_team?key=")
                .concat(key)
                .into_string();

            let _ = db.send_notification(&user, format!("Вас приглашают в команду {}", sender_team), format!("<a href=\"{}\">Присоединиться</a>", link)).await;
            let email = match db.get::<RegistrationData>("login", &user).await {
                Ok(u) => u.unwrap(),
                Err(_) => return Status::InternalServerError
            };

            let _ = mail::send(email.email(), email_header, link);
            Status::Ok
        }
    }
}

#[get("/join_team?<key>")]
pub async fn join_team(db: &State<MongoDriver>, key: String) -> Status {
    let data = jsonwebtoken::decode::<InvitationData>(
        &key,
        &DecodingKey::from_secret(from_config("jwt_secret").as_bytes()),
        &Validation::default(),
    );

    match data {
        Ok(data) => {
            let data = data.claims;
            match db.add_team_member(&data.team, &data.usr).await {
                Ok(_) => Status::Ok,
                Err(_) => Status::InternalServerError
            }
        }
        Err(_) => Status::Forbidden
    }
}

#[get("/check_notifications")]
pub async fn check_notifications(token: Token, db: &State<MongoDriver>) -> Result<String, Status> {
    match db.check_notifications(token.claims.iss).await {
        Err(_) => Err(Status::InternalServerError),
        Ok(notifications) => Ok(serde_json::to_string(&notifications).unwrap())
    }
}

#[get("/leave_team?<team>")]
pub async fn leave_team(token: Token, team: String, db: &State<MongoDriver>) -> Status {
    let user = token.claims.iss;
    match db.remove_team_member(&team, &user).await {
        Ok(_) => Status::Ok,
        Err(_) => Status::InternalServerError
    }
}

#[get("/remove_from_team?<team>&<user>")]
pub async fn remove_from_team(token: Token, team: String, user: String, db: &State<MongoDriver>) -> Status {
    let captain = token.claims.iss;
    match db.check_is_captain(&team, &captain).await {
        Ok(false) => return Status::Forbidden,
        Err(_) => return Status::InternalServerError,
        _ => {}
    }

    match db.remove_team_member(&team, &user).await {
        Ok(_) => Status::Ok,
        Err(_) => Status::InternalServerError
    }
}

#[post("/create_task", data = "<task>")]
pub async fn create_task(token: Token, task: Task, db: &State<MongoDriver>) -> Status {
    let captain = token.claims.iss;
    let sender_team = db.get_user_team(TeamType::Hackathon, &captain).await;
    let sender_team = match sender_team {
        Ok(team) => team,
        Err(GetTeamError::NotInTeam) => return Status::Forbidden,
        _ => return Status::InternalServerError
    };
    match db.check_is_captain(&sender_team, &captain).await {
        Ok(false) => return Status::Forbidden,
        Err(_) => return Status::InternalServerError,
        _ => {}
    }

    match db.create_task(task, &sender_team).await {
        Ok(_) => Status::Ok,
        Err(_) => Status::InternalServerError
    }
}

#[post("/update_task?<team>", data = "<task>")]
pub async fn update_task(token: Token, team: String, task: Task, db: &State<MongoDriver>) -> Status {
    let captain = token.claims.iss;
    match db.check_is_captain(&team, &captain).await {
        Ok(false) => return Status::Forbidden,
        Err(_) => return Status::InternalServerError,
        _ => {}
    }

    match db.update_task(&team, task).await {
        Ok(_) => Status::Ok,
        Err(_) => Status::InternalServerError
    }
}

#[post("/update_task_status?<team>", data = "<task>")]
pub async fn update_task_status(token: Token, team: String, task: Task, db: &State<MongoDriver>) -> Status {
    let user = token.claims.iss;
    let db_team = db.get_team::<Team>("name", &team).await;

    let db_team = match db_team {
        Ok(Some(t)) => t,
        _ => return Status::InternalServerError
    };

    if db_team.captain == user || db_team.members.contains(&user) {
        match db.update_task(&team, task).await {
            Ok(_) => Status::Ok,
            Err(_) => Status::InternalServerError
        }
    } else {
        Status::Forbidden
    }
}

#[post("/remove_task?<team>", data = "<task>")]
pub async fn remove_task(token: Token, team: String, task: Task, db: &State<MongoDriver>) -> Status {
    let captain = token.claims.iss;
    match db.check_is_captain(&team, &captain).await {
        Ok(false) => return Status::Forbidden,
        Err(_) => return Status::InternalServerError,
        _ => {}
    }

    match db.remove_task(&team, task).await {
        Ok(_) => Status::Ok,
        Err(_) => Status::InternalServerError
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