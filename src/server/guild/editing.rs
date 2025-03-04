use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::options::ReplaceOptions;
use serde_json::{Map, Value};
use tokio::sync::{Mutex, RwLock};
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use crate::context::Context;
use crate::models::config::GuildConfig;
use crate::server::guild::ws::{Connection, OutboundAction, OutboundMessage};

struct GuildEditingState {
    pub connections: Vec<Arc<Connection>>,
    pub changes: Value,
    pub edited_by: BTreeSet<Id<UserMarker>>
}

impl Default for GuildEditingState {
    fn default() -> Self {
        Self {
            connections: vec![],
            changes: Value::Object(Map::new()),
            edited_by: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct GuildsEditing(RwLock<HashMap<Id<GuildMarker>, Arc<Mutex<GuildEditingState>>>>);

impl GuildsEditing {
    pub async fn add_connection(&self, guild_id: Id<GuildMarker>, connection_data: Connection) {
        let guild = match self.get_guild(guild_id).await {
            Some(guild) => guild,
            None => {
                let guild = Arc::new(Mutex::new(GuildEditingState::default()));
                let mut lock = self.0.write().await;
                lock.insert(guild_id, guild.clone());
                drop(lock);
                guild
            }
        };

        guild.lock().await.connections.push(Arc::new(connection_data));
    }

    pub async fn remove_connection(&self, guild_id: Id<GuildMarker>, session_id: ObjectId) -> Option<()> {
        let guild = self.get_guild(guild_id).await?;
        let mut lock = guild.lock().await;
        lock.connections = lock.connections
            .iter()
            .filter(|connection| connection.session_id == session_id)
            .cloned()
            .collect();
        Some(())
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
        let config = context.mongodb
            .get_config(guild_id)
            .await
            .inspect_err(|error| println!("{error:?}"))
            .ok()?;

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

    pub async fn apply_changes(&self, context: &Arc<Context>, guild_id: Id<GuildMarker>) -> Option<()> {
        let config = context.mongodb.get_config(guild_id).await
            .inspect_err(|error| println!("{error:?}"))
            .ok()?;

        let is_guild_premium = config.premium;

        let guild = self.get_guild(guild_id).await?;
        let mut guild_lock = guild.lock().await;

        let mut new_config = serde_json::to_value(config)
            .inspect_err(|error| println!("{error:?}"))
            .ok()?;
        json_patch::merge(&mut new_config, &guild_lock.changes);
        let new_config: GuildConfig = serde_json::from_value(new_config)
            .inspect_err(|error| println!("{error:?}"))
            .ok()?;

        if new_config.guild_id != guild_id {
            println!("Someone tried changing guild_id in config id={guild_id}");
            return None
        }

        if new_config.premium != is_guild_premium {
            println!("Someone tried changing premium in config id={guild_id}");
            return None
        }

        context.mongodb.configs
            .replace_one(
                doc! { "guild_id": guild_id.to_string() },
                new_config,
                ReplaceOptions::builder().upsert(true).build()
            )
            .await
            .inspect_err(|error| println!("{error:?}"))
            .ok()?;
        context.mongodb.configs_cache.remove(&guild_id);

        guild_lock.changes = Value::Object(Map::new());
        guild_lock.edited_by.clear();

        Some(())
    }
}
