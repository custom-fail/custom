use crate::models::config::automod::actions::Actions;
use crate::models::config::automod::filters::Filters;
use serde::{Serialize, Deserialize};
use crate::models::config::automod::checks::Checks;
use crate::models::config::moderation::Ignore;

pub mod actions;
pub mod checks;
pub mod filters;
pub mod bucket;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutoModeratorV2 {
    pub filters: Vec<Filters>,
    pub checks: Vec<Checks>,
    pub actions: Vec<Actions>,
    pub ignore: Vec<Ignore>,
    pub reason: String
}