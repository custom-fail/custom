use mongodb::bson::DateTime;
use twilight_model::id::Id;
use twilight_model::id::marker::{ApplicationMarker, GuildMarker, UserMarker};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Task {
    execute_at: DateTime,
    guild_id: Id<GuildMarker>,
    application_id: Option<Id<ApplicationMarker>>,
    action: TaskAction
}

#[derive(Serialize, Deserialize)]
pub enum TaskAction {
    RemoveMuteRole(Id<UserMarker>)
}