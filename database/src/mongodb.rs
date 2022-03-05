use std::collections::HashMap;
use std::sync::Arc;
use mongodb::{Client, Collection, Database};
use mongodb::bson::doc;
use tokio::sync::Mutex;
use twilight_model::channel::embed::Embed;
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, UserMarker};
use crate::models::case::Case;
use crate::models::config::GuildConfig;

pub struct MongoDBConnection {
    pub client: Client,
    pub database: Database,
    pub cases: Collection<Case>,
    pub configs: Collection<GuildConfig>,
    pub configs_cache: Arc<Mutex<HashMap<Id<GuildMarker>, GuildConfig>>>
}

impl MongoDBConnection {

    pub async fn connect(url: String) -> Result<Self, mongodb::error::Error> {

        let client = Client::with_uri_str(url).await?;
        let db = client.database("custom");
        let configs = db.collection::<GuildConfig>("configs");
        let cases = db.collection::<Case>("cases");

        Ok(Self {
            configs_cache: Arc::new(Mutex::new(HashMap::new())),
            database: db,
            cases,
            client,
            configs
        })
    }

    pub async fn get_config(&self, guild_id: Id<GuildMarker>) -> Result<GuildConfig, String> {

        let configs_cache = self.configs_cache.lock().await;
        let config = configs_cache.get(&guild_id);

        if config.is_some() { return Ok(config.unwrap().clone()) };

        let config_db = self.configs.clone_with_type().find_one(doc! { "guild_id": guild_id.to_string() }, None).await;

        match config_db {
            Ok(config_db) => match config_db {
                Some(config_db) => Ok(config_db),
                None => return Err("stop".to_string())
            },
            Err(err) => return Err(format!("{err}"))
        }

    }

    pub async fn create_case(&self, discord_http: Arc<twilight_http::Client>, case: Case, case_embed: Embed, dm_case: Option<Id<UserMarker>>, logs: Option<Id<ChannelMarker>>) -> Result<(), String> {

        self.cases.insert_one(case, None).await.map_err(|err| format!("{err}"))?;

        if let Some(channel_id) = logs {
            discord_http.create_message(channel_id)
                .embeds(&[case_embed.clone()]).map_err(|err| err.to_string())?
                .exec().await.map_err(|err| err.to_string())?
                .model().await.map_err(|err| err.to_string())?;
        }

        if let Some(member_id) = dm_case {
            let channel = discord_http.create_private_channel(member_id)
                .exec().await.map_err(|err| err.to_string())?
                .model().await.map_err(|err| err.to_string())?;
            discord_http.create_message(channel.id)
                .embeds(&[case_embed]).map_err(|err| err.to_string())?
                .exec().await.map_err(|err| err.to_string())?
                .model().await.map_err(|err| err.to_string())?;
        }

        Ok(())

    }

    pub async fn get_next_case_index(&self, guild_id: Id<GuildMarker>) -> Result<u64, String> {
        Ok(self.cases.count_documents(
        doc! { "guild_id": guild_id.to_string() }, None
        ).await.map_err(|err| format!("{err}"))? + 1)
    }

}

impl Clone for MongoDBConnection {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            database: self.database.clone(),
            configs: self.configs.clone_with_type(),
            configs_cache: self.configs_cache.clone(),
            cases: self.cases.clone_with_type()
        }
    }
}