use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Moderation {
    pub mute_type: u8,
    pub native_support: bool
}

impl Moderation {
    fn clone(&self) -> Self {
        Self {
            mute_type: self.mute_type.clone(),
            native_support: self.native_support.clone()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Levels {
    pub xp_timeout: u16,
    pub xp_min: u8,
    pub xp_max: u8
}

impl Clone for Levels {
    fn clone(&self) -> Self {
        Self {
            xp_timeout: self.xp_timeout.clone(),
            xp_min: self.xp_min.clone(),
            xp_max: self.xp_max.clone()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Top {
    pub week: bool,
    pub day: bool,
    pub webhook_url: String
}

impl Clone for Top {
    fn clone(&self) -> Self {
        Self {
            week: self.week.clone(),
            day: self.day.clone(),
            webhook_url: self.webhook_url.clone()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GuildConfig {
    pub guild_id: String,
    pub enabled: HashMap<String, bool>,
    pub moderation: Moderation,
    pub premium: bool,
    pub levels: Levels,
    pub top: Top
}

impl Clone for GuildConfig {
    fn clone(&self) -> Self {
        Self {
            guild_id: self.guild_id.clone(),
            enabled: self.enabled.clone(),
            moderation: self.moderation.clone(),
            premium: self.premium.clone(),
            levels: self.levels.clone(),
            top: self.top.clone()
        }
    }
}