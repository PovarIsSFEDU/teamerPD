use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Team {
    pub name: String,
    pub logo: Option<String>,
    pub captain: String,
    pub members: Vec<String>
}

impl Team {
    pub fn new(name: String, captain: String) -> Self {
        Team {
            name,
            logo: None,
            captain: captain.clone(),
            members: vec![captain]
        }
    }
}