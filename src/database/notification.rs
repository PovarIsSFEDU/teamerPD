use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Notification {
    pub header: String,
    pub body: String,
    pub seen: bool,
    #[serde(rename = "_id")]
    pub id: i32
}

impl Notification {
    pub fn new(header: String, body: String) -> Self {
        Notification {
            header,
            body,
            seen: false,
            id: 0
        }
    }
}