pub mod mongo;
mod user;
mod new_user;
mod team;

pub use mongo::MongoDriver;
pub use user::User;

pub enum LoginError {
    NotExist,
    IncorrectPassword,
    Other
}

pub enum RegistrationResult {
    Ok,
    Exists,
    Other
}

pub enum VerificationError {
    AlreadyVerified,
    Other
}

pub enum DatabaseError {
    NotFound,
    Other
}

pub enum UserDataType {
    Photo,
    TeamName,
    Resume
}

pub enum TeamDataType {
    Name,
    Logo
}

pub enum TeamCreationError {
    Exists,
    Other
}

pub enum GetTeamError {
    NotInTeam,
    NotFound,
    Other
}