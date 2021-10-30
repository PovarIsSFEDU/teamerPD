pub mod mongo;

use crate::auth::LoginData;
pub use mongo::MongoDriver;

pub enum LoginResult {
    Ok,
    NotExist,
    IncorrectPassword,
    Other
}

pub enum RegisterResult {
    Ok,
    Exists,
    Other
}

///Temporary, until we know which db to use
#[async_trait]
pub trait DatabaseDriver: Send + Sync {
    async fn validate_registration(&self, data: &LoginData) -> RegisterResult;
    async fn register(&self, data: LoginData) -> RegisterResult;
    async fn validate_login(&self, data: &LoginData) -> LoginResult;
}