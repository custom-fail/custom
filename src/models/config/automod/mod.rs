use std::collections::HashMap;

use crate::models::config::automod::filters::Filter;
use serde::{Serialize, Deserialize};
use twilight_model::id::Id;
use twilight_model::id::marker::ChannelMarker;
use self::actions::ActionMetadata;
use self::bucket::BucketAction;
use self::ignore::Ignore;
use crate::models::config::automod::checks::Check;

pub mod actions;
pub mod checks;
pub mod filters;
pub mod bucket;
pub mod ignore;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutoModeration {
    pub rules: Vec<AutoModerationRule>,
    pub bucket_actions: HashMap<String, BucketAction>,
    pub logs_channel: Option<Id<ChannelMarker>>,
    pub ignore: Option<Ignore>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BasicAutoModerationRule {
    ScamLinks = 1,
    Mentions = 2,
    CapsLock = 3,
    Invites = 4
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutoModerationRule {
    pub basic_type: Option<BasicAutoModerationRule>,
    pub filters: Vec<Filter>,
    pub checks: Vec<Check>,
    pub actions: Vec<ActionMetadata>,
    pub ignore: Option<Ignore>,
    pub reason: String,
    pub name: String
}