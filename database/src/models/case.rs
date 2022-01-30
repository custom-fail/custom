use twilight_model::datetime::Timestamp;
use serde::{Serialize, Deserialize};
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};

#[derive(Serialize, Deserialize, Debug)]
pub struct Case {
    pub moderator_id: Id<UserMarker>,
    pub created_at: Timestamp,
    pub guild_id: Id<GuildMarker>,
    pub member_id: Id<UserMarker>,
    pub action: u8,
    pub reason: Option<String>,
    pub removed: bool,
    pub duration: Option<String>,
    pub index: u16
}