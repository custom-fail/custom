use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Actions {
    DirectMessage,
    IncreaseBucket,
    DeleteMessage,
    SendLogs,
    Timeout,
    Kick,
    Ban
}