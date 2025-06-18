use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use twilight_model::id::Id;
use twilight_model::id::marker::{ApplicationMarker, GuildMarker};
use crate::models::config::activity::{Levels, Top};
use crate::models::config::moderation::Moderation;

use self::automod::actions::BucketAction;

pub mod moderation;
pub mod activity;
pub mod automod;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuildConfig {
    pub guild_id: Id<GuildMarker>,
    pub application_id: Option<Id<ApplicationMarker>>,
    pub moderation: Option<Moderation>,
    pub premium: bool,
    pub levels: Option<Levels>,
    pub top: Option<Top>
}

impl GuildConfig {
    pub fn new(guild_id: Id<GuildMarker>) -> Self {
        Self {
            guild_id,
            application_id: None,
            moderation: None,
            premium: false,
            levels: None,
            top: None
        }
    }

    pub fn get_bucket_action(&self, key: &str) -> Option<BucketAction> {
        self.moderation.as_ref()?.automod.as_ref().map(|a| a.bucket_actions.get(key).cloned())?
    }
}
