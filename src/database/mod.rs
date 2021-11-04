pub mod mongo;
mod user;

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