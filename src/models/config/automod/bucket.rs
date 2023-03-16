use serde::{Serialize, Deserialize};
use crate::models::config::automod::actions::Action;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "count")]
pub enum IncreaseBucketAmount {
    Stickers,
    Attachments,
    Mentions,
    Static(u8)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BucketAction {
    pub amount: IncreaseBucketAmount,
    pub actions: Vec<Action>,
    pub reason: String,
    // minimal value required to run action
    pub min: u8
}