use crate::models::config::automod::actions::Actions;
use crate::models::config::automod::filters::Filters;
use serde::{Serialize, Deserialize};
use crate::models::config::automod::checks::Checks;
use crate::models::config::moderation::Ignore;

pub mod actions;
pub mod checks;
pub mod filters;
pub mod bucket;

pub enum BasicAutoModerationRule {
    ScamLinks = 1,
    Mentions = 2,
    CapsLock = 3,
    Invites = 4
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutoModeratorRule {
    pub basic_type: Option<BasicAutoModerationRule>,
    pub filters: Vec<Filters>,
    pub checks: Vec<Checks>,
    pub actions: Vec<Actions>,
    pub ignore: Vec<Ignore>,
    pub reason: String,
    pub name: String
}