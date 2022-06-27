use crate::models::config::automod::actions::Action;
use crate::models::config::automod::filters::Filter;
use serde::{Serialize, Deserialize};
use crate::models::config::automod::checks::Check;
use crate::models::config::moderation::Ignore;
use serde_repr::{Serialize_repr, Deserialize_repr};

pub mod actions;
pub mod checks;
pub mod filters;
pub mod bucket;

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum BasicAutoModerationRule {
    ScamLinks = 1,
    Mentions = 2,
    CapsLock = 3,
    Invites = 4
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutoModeratorRule {
    pub basic_type: Option<BasicAutoModerationRule>,
    pub filters: Vec<Filter>,
    pub checks: Vec<Check>,
    pub actions: Vec<Action>,
    pub ignore: Vec<Ignore>,
    pub reason: String,
    pub name: String
}