use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use crate::models::config::activity::{Levels, Top};
use crate::models::config::automod::bucket::BucketActions;
use crate::models::config::moderation::Moderation;

pub mod moderation;
pub mod activity;
pub mod automod;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuildConfig {
    pub guild_id: Id<GuildMarker>,
    pub enabled: HashMap<String, bool>,
    pub moderation: Moderation,
    pub premium: bool,
    pub levels: Levels,
    pub top: Top
}

impl GuildConfig {
    pub fn default(guild_id: Id<GuildMarker>) -> Self {
        Self {
            guild_id,
            enabled: HashMap::new(),
            moderation: Moderation {
                mute_type: 0,
                native_support: false,
                logs_channel: None,
                dm_case: false,
                automod_logs: None,
                bucket_actions: HashMap::new(),
                automod: vec![],
                automod_ignore: vec![]
            },
            premium: false,
            levels: Levels {
                xp_timeout: 0,
                xp_min: 0,
                xp_max: 0
            },
            top: Top {
                week: false,
                day: false,
                webhook_url: "".to_string()
            }
        }
    }

    pub fn get_bucket_action(&self, key: String) -> Option<BucketActions> {
        self.moderation.bucket_actions.get(key.as_str()).cloned()
    }

}

