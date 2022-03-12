use serde::{Serialize, Deserialize};
use crate::models::config::automod::actions::Actions;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BucketActions {
    pub actions: Vec<Actions>,
    // minimal value required to run action
    pub min: u8
}