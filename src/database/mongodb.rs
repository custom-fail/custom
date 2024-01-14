use std::sync::Arc;
use dashmap::DashMap;
use futures_util::TryStreamExt;
use mongodb::{Client, Collection, Database};
use mongodb::bson::doc;
#[cfg(feature = "tasks")]
use mongodb::bson::DateTime;
use twilight_model::channel::message::Embed;
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, UserMarker};
#[cfg(any(feature = "tasks", feature = "custom-clients"))]
use crate::gateway::clients::ClientData;
use crate::models::case::Case;
use crate::models::config::GuildConfig;
use crate::models::task::Task;
use crate::database::redis::RedisConnection;
use crate::utils::errors::Error;

#[derive(Clone)]
pub struct MongoDBConnection {
    pub client: Client,
    pub database: Database,
    pub cases: Collection<Case>,
    pub configs: Collection<GuildConfig>,
    #[cfg(any(feature = "tasks", feature = "custom-clients"))]
    pub clients: Collection<ClientData>,
    pub tasks: Collection<Task>,
    pub configs_cache: Arc<DashMap<Id<GuildMarker>, GuildConfig>>
}

impl MongoDBConnection {

    pub async fn connect(uri: String) -> Result<Self, mongodb::error::Error> {
        let client = Client::with_uri_str(uri).await?;
        let db = client.database("custom");
        let configs = db.collection::<GuildConfig>("configs");
        let cases = db.collection("cases");
        #[cfg(any(feature = "tasks", feature = "custom-clients"))]
        let clients = db.collection("clients");
        let tasks = db.collection("tasks");

        Ok(Self {
            configs_cache: Arc::new(DashMap::new()),
            database: db,
            cases,
            client,
            #[cfg(any(feature = "tasks", feature = "custom-clients"))]
            clients,
            configs,
            tasks
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
                ).await?.unwrap_or_else(|| GuildConfig::new(guild_id));

                self.configs_cache.insert(guild_id, config.to_owned());

                Ok(config)
            }
        }

    }

    pub async fn create_case(
        &self,
        discord_http: Arc<twilight_http::Client>,
        redis: &RedisConnection,
        case: Case,
        case_embed: Embed,
        dm_case: Option<Id<UserMarker>>,
        logs: Option<Id<ChannelMarker>>
    ) -> Result<(), Error> {

        self.cases.insert_one(case.to_owned(), None).await.map_err(Error::from)?;

        if let Some(channel_id) = logs {
            discord_http.create_message(channel_id)
                .embeds(&[case_embed.clone()]).map_err(Error::from)?
                .await.map_err(Error::from)?
                .model().await.map_err(Error::from)?;
        }

        if let Some(member_id) = dm_case {
            let channel = discord_http.create_private_channel(member_id)
                .await.map_err(Error::from)?
                .model().await.map_err(Error::from)?;
            let embed = case.to_dm_embed(redis).map_err(Error::from)?;
            discord_http.create_message(channel.id)
                .embeds(&[embed]).map_err(Error::from)?
                .await.map_err(Error::from)?
                .model().await.map_err(Error::from)?;
        }

        Ok(())

    }

    pub async fn get_next_case_index(&self, guild_id: Id<GuildMarker>) -> Result<u64, Error> {
        Ok(self.cases.count_documents(
            doc! { "guild_id": guild_id.to_string() }, None
        ).await.map_err(Error::from)? + 1)
    }

    pub async fn create_task(&self, task: Task) -> Result<(), Error> {
        self.tasks.insert_one(task, None).await.map(|_| ()).map_err(Error::from)
    }

    #[cfg(feature = "tasks")]
    pub async fn get_and_delete_future_tasks(&self, after: u64) -> Result<Vec<Task>, Error> {
        let time = DateTime::from_millis(DateTime::now().timestamp_millis() + after as i64);
        let filter = doc! { "execute_at": { "$lt": time } };
        let tasks = self.tasks.find(filter.to_owned(), None)
            .await.map_err(Error::from)?.try_collect().await.map_err(Error::from);
        self.tasks.delete_many(filter, None).await.map_err(Error::from)?;
        tasks
    }
}