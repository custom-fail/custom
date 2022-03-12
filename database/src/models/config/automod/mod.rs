use crate::models::config::automod::actions::Actions;
use crate::models::config::automod::filters::Filters;
use serde::{Serialize, Deserialize};
use crate::models::config::automod::checks::Checks;

pub mod actions;
pub mod checks;
pub mod filters;
pub mod bucket;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AntiCapsLock {
    pub max_uppercase: u8,
    pub min_msg_len: u16,
    pub max_msg_len: u16,
    pub first_action: String,
    pub repeat_action: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AntiInvites {
    pub allowed_invites: Vec<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutoModeratorV2 {
    pub filters: Vec<Filters>,
    pub checks: Vec<Checks>,
    pub actions: Vec<Actions>
}