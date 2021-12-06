use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    pub logo: Option<String>,
    pub captain: String,
    pub members: Vec<String>
}