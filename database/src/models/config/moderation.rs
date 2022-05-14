use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, RoleMarker, UserMarker};
use crate::models::config::automod::AutoModeratorV2;
use crate::models::config::automod::bucket::BucketActions;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Ignore {
    Channel(Id<ChannelMarker>),
    Role(Id<RoleMarker>),
    User(Id<UserMarker>)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Moderation {
    pub mute_type: u8,
    pub native_support: bool,
    pub logs_channel: Option<Id<ChannelMarker>>,
    pub dm_case: bool,
    pub automod_logs: Option<Id<ChannelMarker>>,
    pub bucket_actions: HashMap<String, BucketActions>,
    pub automod: Vec<AutoModeratorV2>,
    pub automod_ignore: Vec<Ignore>
}