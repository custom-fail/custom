use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use crate::models::config::activity::{Levels, Top};
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