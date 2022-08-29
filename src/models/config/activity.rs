use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Levels {
    pub xp_timeout: u16,
    pub xp_min: u8,
    pub xp_max: u8
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Top {
    pub week: bool,
    pub day: bool,
    pub webhook_url: String
}