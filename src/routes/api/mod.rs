use crate::auth::{token, LoginData, RegistrationData};
use crate::database::{DatabaseError, LoginResult, MongoDriver, RegisterResult, VerificationError};
use crate::{crypto, mail, DOMAIN};
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::State;

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
        RegisterResult::Exists =>
            return Custom(Status::BadRequest, "User with given login or email already exists".to_owned()),

        RegisterResult::Other =>
            return Custom(Status::InternalServerError, "Internal Server Error".to_owned()),

        RegisterResult::Ok => {}
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

#[get("/send_verification?<user>")]
pub async fn send_verification_link(user: String, db: &State<MongoDriver>) -> Result<Status, Custom<&str>> {
    let key = db.get_verification_key(user).await;
    match key {
        Ok(key) => {
            let mut uri = DOMAIN.to_owned();
            uri.push_str("/verify?key=");
            uri.push_str(key.1.as_str());
            //Uncomment when SMTP is working
            //mail::send_email_verification(key.0, uri);
            Ok(Status::Ok)
        }

        Err(DatabaseError::NotFound) => Err(Custom(Status::NotFound, "User or key not found")),

        Err(DatabaseError::Other) => {
            Err(Custom(Status::InternalServerError, "Internal Server Error"))
        }
    }
}

#[post("/recover?<key>", data = "<user>")]
pub async fn recover_password(key: String, user: LoginData, db: &State<MongoDriver>) -> Status {
    let result = db.validate_recovery(key, user).await;

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
            let key = crypto::hash(key.as_bytes());

            let mut link = DOMAIN.to_owned();
            link.push_str("/recover?key=");
            link.push_str(key.as_str());

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
