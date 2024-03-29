use std::collections::HashMap;

use crate::models::config::automod::filters::FilterMetadata;
use serde::{Serialize, Deserialize};
use twilight_model::id::Id;
use twilight_model::id::marker::ChannelMarker;
use self::actions::{ActionMetadata, BucketAction};
use self::ignore::Ignore;
use crate::models::config::automod::checks::Check;

pub mod actions;
pub mod checks;
pub mod filters;
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
    pub check_on_edit: bool,
    pub filters: Vec<FilterMetadata>,
    pub checks: Vec<Check>,
    pub actions: Vec<ActionMetadata>,
    pub ignore: Option<Ignore>,
    pub reason: String,
    pub name: String,
}

#[derive(PartialEq)]
pub enum TrigerEvent {
    MessageCreate,
    MessageUpdate
}