use mongodb::bson::DateTime;
use twilight_model::id::Id;
use twilight_model::id::marker::{ApplicationMarker, GuildMarker, UserMarker};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub execute_at: DateTime,
    pub guild_id: Id<GuildMarker>,
    pub action: TaskAction
}

#[derive(Serialize, Deserialize)]
pub enum TaskAction {
    RemoveMuteRole(Id<UserMarker>)
}