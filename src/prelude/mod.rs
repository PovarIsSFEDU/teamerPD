pub mod request;
pub mod concat;

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

pub fn get_ext(name: &String) -> String {
    name
        .split('.')
        .collect::<Vec<&str>>()
        .last()
        .unwrap()
        .to_lowercase()
}

pub fn is_image_ext(ext: &str) -> bool {

    matches_extension(ext, ["jpg", "jpeg", "png", "gif"])
}

pub fn is_doc_ext(ext: &str) -> bool {
    matches_extension(ext, ["doc", "docx", "pdf"])
}

pub trait MapBoth<T, E> {
    fn map_both<F, U, F1, E1>(self, ok: F, err: F1) -> Result<U, E1>
        where F: FnOnce(T) -> U,
              F1: FnOnce(E) -> E1;
}

impl<T, E> MapBoth<T, E> for Result<T, E> {
    fn map_both<F, U, F1, E1>(self, ok: F, err: F1) -> Result<U, E1>
        where F: FnOnce(T) -> U,
              F1: FnOnce(E) -> E1
    {
        match self {
            Ok(res) => Ok(ok(res)),
            Err(res) => Err(err(res))
        }
    }
}

fn matches_extension(ext: &str, exts: impl IntoIterator<Item = &'static str>) -> bool {
    for e in IntoIterator::into_iter(exts) {
        if e == ext {
            return true;
        }
    }

    false
}