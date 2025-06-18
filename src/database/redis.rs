use std::str::FromStr;
use futures_util::StreamExt;
use redis::{Client, RedisError};
use serde_json::json;
use twilight_model::id::marker::{GuildMarker, RoleMarker, UserMarker};
use twilight_model::id::Id;
use twilight_model::util::ImageHash;
use serde::{Serialize, Deserialize};
use crate::utils::errors::Error;
use redis::AsyncCommands;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::error::SendError;
use tracing::info;
use crate::database::mongodb::MongoDBConnection;

#[derive(Serialize, Deserialize, Debug)]
pub struct PartialGuild {
    pub name: String,
    pub icon: Option<ImageHash>,
    pub roles: Vec<Id<RoleMarker>>
}

#[derive(Clone)]
pub struct RedisConnection {
    pub client: Client,
    #[cfg(feature = "api")]
    pub pub_sub_tx: UnboundedSender<Id<GuildMarker>>
}

macro_rules! connection {
    ($self: expr) => {
        $self.client.get_multiplexed_async_connection().await
    };
}

impl RedisConnection {
    pub fn connect(url: String) -> Result<Self, RedisError> {
        let client = Client::open(url)?;

        #[cfg(feature = "api")]
        let (tx, mut rx) =
            tokio::sync::mpsc::unbounded_channel::<Id<GuildMarker>>();

        #[cfg(feature = "api")]
        {
            let client = client.to_owned();
            tokio::spawn(async move {
                while let Some(guild_id) = rx.recv().await {
                    client
                        .get_multiplexed_async_connection()
                        .await
                        .expect("Cannot get redis connection")
                        .publish("configs", guild_id.to_string())
                        .await
                        .expect("Error while sending pubsub message")
                }
            });

        }

        Ok(Self { client, #[cfg(feature = "api")] pub_sub_tx: tx })
    }

    pub async fn set_guild(&self, id: Id<GuildMarker>, guild: PartialGuild) -> Result<(), RedisError> {
        let mut connection = connection!(self)?;
        let data = json!(guild).to_string();
        connection.set(format!("guilds.{id}"), data).await
    }

    pub async fn get_guild(&self, id: Id<GuildMarker>) -> Result<PartialGuild, Error> {
        let mut connection = connection!(self).map_err(Error::from)?;
        let data: String = connection.get(format!("guilds.{id}")).await.map_err(Error::from)?;
        serde_json::from_str(data.as_str()).map_err(Error::from)
    }

    pub async fn delete_guild(&self, id: Id<GuildMarker>) -> Result<(), RedisError> {
        let mut connection = connection!(self)?;
        connection.del(format!("guilds.{id}")).await
    }

    pub async fn get_by_position(
        &self,
        path: String,
        position: usize,
    ) -> Result<Option<u32>, RedisError> {
        let mut connection = connection!(self)?;
        let result: Vec<u32> = connection.zrevrange_withscores(
            path,
            (position - 1) as isize,
            (position - 1) as isize,
        ).await?;
        Ok(result.first().cloned())
    }

    pub async fn get_by_user(
        &self,
        path: String,
        user_id: Id<UserMarker>,
    ) -> Result<(u32, u32), RedisError> {
        let mut connection = connection!(self)?;
        let user_id = user_id.to_string();
        let score = connection.zscore(path.clone(), user_id.clone()).await?;
        let position = connection.zrevrank(path, user_id).await?;
        Ok((score, position))
    }

    pub async fn guild_exists(&self, id: Id<GuildMarker>) -> Result<bool, RedisError> {
        let mut connection = connection!(self)?;
        connection.exists(format!("guilds.{id}")).await
    }

    pub async fn get_all(&self, path: String, limit: isize) -> Result<Vec<(String, u32)>, RedisError> {
        let mut connection = connection!(self)?;
        connection.zrevrange_withscores(path, 0, limit - 1).await
    }

    pub async fn increase(
        &self,
        path: String,
        user_id: Id<UserMarker>,
        count: u8,
    ) -> Result<(), RedisError> {
        let mut connection = connection!(self)?;
        connection.zincr(path, user_id.to_string(), count).await
    }

    pub async fn watch_config_updates(
        &self,
        mongodb: &MongoDBConnection
    ) -> Result<(), RedisError> {
        let mut pubsub = self.client.get_async_pubsub().await?;
        pubsub.subscribe("configs").await?;

        while let Some(message) = pubsub.on_message().next().await {
            const ERROR: &str = "Received invalid payload instead of guild id";
            let id: String = message.get_payload().expect(ERROR);
            let id: Id<GuildMarker> = Id::from_str(id.as_str()).expect(ERROR);
            mongodb.configs_cache.remove(&id);

            info!(
                name: "removed config from cache due to update",
                cache_size = mongodb.configs_cache.len(),
                guild_id = %id,
            );
        }

        Ok(())
    }

    #[cfg(feature = "api")]
    pub async fn announce_config_update(
        &self,
        guild_id: Id<GuildMarker>
    ) -> Result<(), SendError<Id<GuildMarker>>> {
        self.pub_sub_tx.send(guild_id)
    }
}
