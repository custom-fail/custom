use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Filters {
    MessageLength(MessageLength),
    Attachments(Attachments),
    Stickers
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageLength {
    pub min: Option<u16>,
    pub max: Option<u16>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attachments {
    pub min: Option<u8>,
    pub max: Option<u8>
}

