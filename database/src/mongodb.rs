use std::collections::HashMap;
use std::sync::Arc;
use mongodb::{Client, Collection, Database};
use mongodb::bson::doc;
use tokio::sync::Mutex;
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
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