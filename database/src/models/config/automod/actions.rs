use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Actions {
    DirectMessage,
    IncreaseBucket,
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