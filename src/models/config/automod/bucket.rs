use serde::{Serialize, Deserialize};
use crate::models::config::automod::actions::Actions;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "count")]
pub enum IncreaseBucketAmount {
    Stickers,
    Attachments,
    Mentions,
    Static(u8)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BucketActions {
    pub amount: IncreaseBucketAmount,
    pub actions: Vec<Actions>,
    pub reason: String,
    // minimal value required to run action
    pub min: u8
}