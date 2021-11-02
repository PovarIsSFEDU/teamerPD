pub mod request;

use rocket::fs::NamedFile;
use rocket::response::content::Html;
use rocket::futures::executor;
use std::path::Path;

pub type Page = Html<Option<NamedFile>>;

pub fn html_from_file(path: &str, name: &str) -> Page {
    let result = executor::block_on(
        NamedFile::open(
            Path::new(path).join(name)
        )
    );

    Html(result.ok())
}