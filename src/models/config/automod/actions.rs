use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionMetadata {
    #[serde(flatten)]
    pub action: Action,
    pub sync: bool
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum Action {
    DirectMessage,
    IncreaseBucket(IncreaseBucket),
    DeleteMessage,
    SendLogs,
    Timeout(Timeout),
    Kick,
    Ban
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "count")]
pub enum IncreaseBucketAmount {
    Stickers,
    Attachments,
    Mentions,
    Static(u8)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IncreaseBucket {
    pub key: String,
    pub amount: IncreaseBucketAmount,
    pub per_channel: bool,
    /// Time before value is decreased (in seconds)
    pub duration: u16
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BucketAction {
    pub actions: Vec<ActionMetadata>,
    pub reason: String,
    /// Minimal value required to run action
    pub limit: u8
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Timeout {
    pub duration: u32
}