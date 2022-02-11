use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, GuildMarker};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Action {
    pub delete_message: bool,
    pub duration: Option<u64>,
    pub r#type: Option<u8>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AutoModeratorType {
    MessageLength = 1,
    AntiCapsLock = 2
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageLength {
    pub max_lines: u16,
    pub line_len: u8,
    pub first_action: String,
    pub repeat_action: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AntiCapsLock {
    pub max_uppercase: u8,
    pub min_msg_len: u16,
    pub max_msg_len: u16,
    pub first_action: String,
    pub repeat_action: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum AutoModerator {
    MessageLength(MessageLength),
    AntiCapsLock(AntiCapsLock)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Moderation {
    pub mute_type: u8,
    pub native_support: bool,
    pub logs_channel: Option<Id<ChannelMarker>>,
    pub dm_case: bool,
    pub automod_actions: HashMap<String, Action>,
    pub automod: Vec<AutoModerator>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Levels {
    pub xp_timeout: u16,
    pub xp_min: u8,
    pub xp_max: u8
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Top {
    pub week: bool,
    pub day: bool,
    pub webhook_url: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuildConfig {
    pub guild_id: Id<GuildMarker>,
    pub enabled: HashMap<String, bool>,
    pub moderation: Moderation,
    pub premium: bool,
    pub levels: Levels,
    pub top: Top
}