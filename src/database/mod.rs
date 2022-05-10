pub mod mongo;
pub mod team;
mod user;
mod new_user;
pub mod notification;
pub mod task;

use mongodb::error::Error;
pub use mongo::MongoDriver;
pub use user::User;

#[allow(dead_code)]
pub enum LoginError {
    NotExist,
    IncorrectPassword,
    Other
}

#[allow(dead_code)]
pub enum RegistrationResult {
    Ok,
    Exists,
    Other
}

#[allow(dead_code)]
pub enum VerificationError {
    AlreadyVerified,
    Other
}

#[allow(dead_code)]
pub enum DatabaseError {
    NotFound,
    Other
}

impl From<mongodb::error::Error> for DatabaseError {
    fn from(_: Error) -> Self {
        Self::Other
    }
}

#[allow(dead_code)]
pub enum UserDataType {
    Photo,
    TeamName,
    Resume,
    Email,
    AdminStatus,
    Competences
}

#[allow(dead_code)]
pub enum TeamDataType {
    Name,
    Logo
}

#[allow(dead_code)]
pub enum TeamCreationError {
    Exists,
    Other
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum GetTeamError {
    NotInTeam,
    NotFound,
    Other
}

#[allow(dead_code)]
pub enum AddUserToTeamResult {
    Ok,
    UserNotFound,
    TeamNotFound,
    Exists,
    Error,
    Other
}