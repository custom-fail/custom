use twilight_model::datetime::Timestamp;
use serde::{Serialize, Deserialize};
use twilight_model::guild::Guild;
use twilight_model::id::Id;
use twilight_model::user::User;

#[derive(Serialize, Deserialize, Debug)]
pub struct Case {
    pub moderator_id: Id<User>,
    pub created_at: Timestamp,
    pub guild_id: Id<Guild>,
    pub member_id: Id<User>,
    pub action: u8,
    pub reason: Option<String>,
    pub removed: bool,
    pub duration: Option<String>,
    pub index: u16
}