use crate::crypto;
use crate::database::{DatabaseError, TeamDataType, UserDataType};
use crate::prelude::*;
use rocket::fs::TempFile;
use std::path::Path;
use rocket::http::Status;
use rocket::response::status::Custom;
use crate::database::mongo::DatabaseOperationResult;

pub(super) use self::UploadDataType::*;

pub(super) enum UploadDataType {
    User(UserDataType),
    Team(TeamDataType),
}

pub(super) fn generate_upload_name(owner: &str, file: &TempFile<'_>) -> String {
    let name = file.name().unwrap().to_owned();
    let ext = get_ext(&name);
    let name = crypto::hash(owner.as_bytes());
    format!("{}.{}", name, ext)
}

pub(super) async fn save_upload(mut file: TempFile<'_>, name: String) -> std::io::Result<()> {
    file.persist_to(Path::new("uploads/").join(name)).await
}

pub trait IntoCustom {
    fn into_custom(self) -> Custom<()>;
}

impl IntoCustom for DatabaseOperationResult {
    fn into_custom(self) -> Custom<()> {
        match self {
            Ok(_) => Custom(Status::Ok, ()),
            Err(DatabaseError::NotFound) => Custom(Status::NotFound, ()),
            Err(DatabaseError::Other) => Custom(Status::InternalServerError, ())
        }
    }
}