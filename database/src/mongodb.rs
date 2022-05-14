use std::sync::Arc;
use dashmap::DashMap;
use mongodb::{Client, Collection, Database};
use mongodb::bson::doc;
use twilight_model::channel::embed::Embed;
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, UserMarker};
use utils::errors::Error;
use crate::models::case::Case;
use crate::models::config::GuildConfig;
use crate::redis::RedisConnection;

#[derive(Clone)]
pub struct MongoDBConnection {
    pub client: Client,
    pub database: Database,
    pub cases: Collection<Case>,
    pub configs: Collection<GuildConfig>,
    pub configs_cache: Arc<DashMap<Id<GuildMarker>, GuildConfig>>
}

impl MongoDBConnection {

    pub async fn connect(url: String) -> Result<Self, mongodb::error::Error> {

        let client = Client::with_uri_str(url).await?;
        let db = client.database("custom");
        let configs = db.collection::<GuildConfig>("configs");
        let cases = db.collection::<Case>("cases");

        Ok(Self {
            configs_cache: Arc::new(DashMap::new()),
            database: db,
            cases,
            client,
            configs
        })
    }

    pub async fn get_config(&self, guild_id: Id<GuildMarker>) -> Result<GuildConfig, mongodb::error::Error> {

        match self.configs_cache.get(&guild_id){
            Some(config) => {
                Ok(config.to_owned())
            },
            None => {
                let config = self.configs.clone_with_type().find_one(
                    doc! {
                        "guild_id": guild_id.to_string()
                    }, None
                ).await?.unwrap_or_else(|| GuildConfig::default(guild_id));

                self.configs_cache.insert(guild_id, config.to_owned());

                Ok(config)
            }
        }

    }

    pub async fn create_case(
        &self,
        discord_http: Arc<twilight_http::Client>,
        redis: RedisConnection,
        case: Case,
        case_embed: Embed,
        dm_case: Option<Id<UserMarker>>,
        logs: Option<Id<ChannelMarker>>
    ) -> Result<(), Error> {

        self.cases.insert_one(case.to_owned(), None).await.map_err(Error::from)?;

        if let Some(channel_id) = logs {
            discord_http.create_message(channel_id)
                .embeds(&[case_embed.clone()]).map_err(Error::from)?
                .exec().await.map_err(Error::from)?
                .model().await.map_err(Error::from)?;
        }

        if let Some(member_id) = dm_case {
            let channel = discord_http.create_private_channel(member_id)
                .exec().await.map_err(Error::from)?
                .model().await.map_err(Error::from)?;
            let embed = case.to_dm_embed(redis).map_err(Error::from)?;
            discord_http.create_message(channel.id)
                .embeds(&[embed]).map_err(Error::from)?
                .exec().await.map_err(Error::from)?
                .model().await.map_err(Error::from)?;
        }

        Ok(())

    }

    pub async fn get_next_case_index(&self, guild_id: Id<GuildMarker>) -> Result<u64, Error> {
        Ok(self.cases.count_documents(
            doc! { "guild_id": guild_id.to_string() }, None
        ).await.map_err(Error::from)? + 1)
    }

}