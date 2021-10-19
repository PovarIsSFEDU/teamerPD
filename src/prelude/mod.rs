use rocket::fs::NamedFile;
use rocket::response::content::Html;
use rocket::futures::executor;
pub type Page = Html<Option<NamedFile>>;

pub fn concat(str1: &str, str2: &str) -> String {
    let mut str = str1.to_owned();
    str.push_str(str2);
    str
}

pub fn html_from_file(path: &str, name: &str) -> Page {
    let result = executor::block_on(
        NamedFile::open(
            concat(path, name)
        )
    );

    Html(result.ok())
}