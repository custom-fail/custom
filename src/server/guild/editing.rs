use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use mongodb::bson::oid::ObjectId;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use crate::context::Context;
use crate::server::guild::ws::{Connection, OutboundAction, OutboundMessage};

struct GuildEditingState {
    pub connections: Vec<Arc<Connection>>,
    pub changes: Value,
    pub edited_by: BTreeSet<Id<UserMarker>>
}

#[derive(Default)]
pub struct GuildsEditing(RwLock<HashMap<Id<GuildMarker>, Arc<Mutex<GuildEditingState>>>>);

impl GuildsEditing {
    pub async fn add_connection(&self, guild_id: Id<GuildMarker>, connection_data: Connection) {
        todo!()
    }

    pub async fn remove_connection(&self, guild_id: Id<GuildMarker>, session_id: ObjectId) {
        todo!()
    }

    pub async fn marge_changes(
        &self,
        author: Id<UserMarker>,
        guild_id: Id<GuildMarker>,
        changes: Value
    ) -> Option<()> {
        let guild = self.get_guild(guild_id).await?;
        let mut guild_lock = guild.lock().await;
        json_patch::merge(&mut guild_lock.changes, &changes);
        guild_lock.edited_by.insert(author);
        Some(())
    }

    async fn get_guild(&self, guild_id: Id<GuildMarker>) -> Option<Arc<Mutex<GuildEditingState>>> {
        let list_lock = self.0.read().await;
        list_lock.get(&guild_id).cloned()
    }

    pub async fn broadcast_changes(&self, context: &Arc<Context>, guild_id: Id<GuildMarker>) -> Option<()> {
        let config = context.mongodb.get_config(guild_id).await.ok()?;
        let guild = self.get_guild(guild_id).await?;
        let guild_lock = guild.lock().await;
        let users = guild_lock.connections
            .iter().map(|connection| connection.user_id)
            .collect::<Vec<Id<UserMarker>>>();

        for connection in &guild_lock.connections {
            let _ = connection.tx.send(OutboundAction::Message(OutboundMessage::UpdateConfigurationData {
                saved_config: config.to_owned(),
                changes: guild_lock.changes.to_owned(),
                users: users.to_owned(),
            }));
        }

        Some(())
    }
}
