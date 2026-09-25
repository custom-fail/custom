use std::collections::HashMap;
use std::sync::Arc;
use json_patch::Patch;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use serde::Serialize;
use tokio::sync::{Mutex, RwLock};
use tracing::{error, warn};
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use crate::context::Context;
use crate::gateway::clients::DiscordClients;
use crate::models::config::GuildConfig;
use crate::server::guild::commands::get_guild_commands_list;
use crate::server::guild::ws::{Connection, OutboundAction, OutboundMessage};

#[derive(Clone, Debug, Serialize)]
pub struct Change {
    pub author_id: Id<UserMarker>,
    pub changes: Patch
}

struct GuildEditingState {
    pub connections: Vec<Arc<Connection>>,
    pub changes: Vec<Change>,
}

impl Default for GuildEditingState {
    fn default() -> Self {
        Self {
            connections: vec![],
            changes: vec![],
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
        author_id: Id<UserMarker>,
        guild_id: Id<GuildMarker>,
        changes: Patch
    ) -> Option<()> {
        let guild = self.get_guild(guild_id).await?;
        let mut guild_lock = guild.lock().await;
        guild_lock.changes.push(Change {
            author_id,
            changes
        });
        Some(())
    }

    async fn get_guild(&self, guild_id: Id<GuildMarker>) -> Option<Arc<Mutex<GuildEditingState>>> {
        let list_lock = self.0.read().await;
        list_lock.get(&guild_id).cloned()
    }

    pub async fn broadcast_users(&self, guild_id: Id<GuildMarker>) -> Option<()> {
        let guild = self.get_guild(guild_id).await?;
        let guild_lock = guild.lock().await;

        let users = guild_lock.connections
            .iter().map(|connection| connection.user_id)
            .collect::<Vec<Id<UserMarker>>>();

        for connection in &guild_lock.connections {
            let _ = connection.tx.send(OutboundAction::Message(OutboundMessage::OverwriteUsers(users.to_owned())));
        }

        Some(())
    }

    pub async fn broadcast_change(
        &self, guild_id: Id<GuildMarker>, author_id: Id<UserMarker>, changes: Patch
    ) -> Option<()> {
        let guild = self.get_guild(guild_id).await?;
        let guild_lock = guild.lock().await;

        for connection in &guild_lock.connections {
            let _ = connection.tx.send(OutboundAction::Message(OutboundMessage::PushChange(Change {
                author_id,
                changes: changes.to_owned()
            })));
        }

        Some(())
    }

    pub async fn get_initialization_data(&self, context: &Arc<Context>, guild_id: Id<GuildMarker>)
        -> Option<(GuildConfig, Vec<Change>, Vec<Id<UserMarker>>)> {
        let config = context.mongodb
            .get_config(guild_id)
            .await
            .inspect_err(|error| error!(name: "mongodb error", ?error))
            .ok()?;

        let guild = self.get_guild(guild_id).await?;
        let guild_lock = guild.lock().await;
        let users = guild_lock.connections
            .iter().map(|connection| connection.user_id)
            .collect::<Vec<Id<UserMarker>>>();

        Some((config.to_owned(), guild_lock.changes.to_owned(), users))
    }

    pub async fn broadcast_config_overwrite(
        &self,
        context: &Arc<Context>,
        guild_id: Id<GuildMarker>,
        is_synced: bool
    ) -> Option<()> {
        let config = context.mongodb
            .get_config(guild_id)
            .await
            .inspect_err(|error| error!(name: "mongodb error", ?error))
            .ok()?;

        let guild = self.get_guild(guild_id).await?;
        let guild_lock = guild.lock().await;

        for connection in &guild_lock.connections {
            let _ = connection.tx.send(OutboundAction::Message(OutboundMessage::OverwriteConfigurationData {
                saved_config: config.to_owned(),
                changes: guild_lock.changes.to_owned(),
                is_synced
            }));
        }

        Some(())
    }

    pub async fn apply_changes(&self, context: &Arc<Context>, guild_id: Id<GuildMarker>) -> Option<bool> {
        let config = context.mongodb.get_config(guild_id).await
            .inspect_err(|error| error!(name: "mongodb error", ?error))
            .ok()?;

        let is_guild_premium = config.premium;

        let guild = self.get_guild(guild_id).await?;
        let mut guild_lock = guild.lock().await;

        let mut new_config = serde_json::to_value(config)
            .inspect_err(|error| error!(name: "cannot convert guild config to value", ?error))
            .ok()?;
        for patch in &guild_lock.changes {
            json_patch::patch(&mut new_config, &patch.changes)
                .inspect_err(|error| error!(name: "error applying patch to guild config", ?patch, ?error))
                .ok()?;
        }

        let new_config: GuildConfig = serde_json::from_value(new_config)
            .inspect_err(|error| error!(name: "cannot serialize config after applying patches", ?error))
            .ok()?;

        if new_config.guild_id != guild_id {
            warn!(name: "someone tried changing guild_id in guild config", %guild_id);
            return None
        }

        if new_config.premium != is_guild_premium {
            warn!(name: "someone tried changing premium in guild config", %guild_id);
            return None
        }

        let is_synced = context.redis.are_commands_synced(&new_config).await
            .inspect_err(|error| error!(name: "redis error", ?error))
            .unwrap_or_else(|_| true);

        context.mongodb.configs
            .replace_one(
                doc! { "guild_id": guild_id.to_string() },
                new_config
            )
            .upsert(true)
            .await
            .inspect_err(|error| error!(name: "mongodb error", ?error))
            .ok()?;
        context.mongodb.configs_cache.remove(&guild_id);

        guild_lock.changes = vec![];

        Some(is_synced)
    }

    pub async fn register_commands(
        &self,
        context: &Arc<Context>,
        discord_http: &Arc<twilight_http::Client>,
        discord_clients: &DiscordClients,
        guild_id: Id<GuildMarker>
    ) -> Option<GuildConfig> {
        let config = context.mongodb.get_config(guild_id).await
            .inspect_err(|error| error!(name: "mongodb error", ?error))
            .ok()?;

        let guild_discord_http = config.application_id
            .and_then(|id| {
                discord_clients.get(&id).map(|option| option.value().clone())
            })
            .unwrap_or_else(|| discord_http.clone());
        let application_id = config.application_id.unwrap_or_else(|| context.application_id);

        let commands = get_guild_commands_list(&config);
        guild_discord_http
            .interaction(application_id)
            .set_guild_commands(guild_id, &commands)
            .await
            .inspect_err(|error| {
                error!(name: "error setting guild commands", ?error, %guild_id, %application_id)
            })
            .ok()?;

        Some(config)
    }
}
