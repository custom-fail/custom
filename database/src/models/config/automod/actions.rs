use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "key")]
pub enum Actions {
    DirectMessage,
    IncreaseBucket(String),
    DeleteMessage,
    SendLogs,
    Timeout(Timeout),
    Kick,
    Ban
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Timeout {
    pub duration: i64
}