pub mod mongo;
mod user;
mod dbinsert;

pub use mongo::MongoDriver;
pub use user::User;

pub enum LoginResult {
    NotExist,
    IncorrectPassword,
    Other
}

pub enum RegisterResult {
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