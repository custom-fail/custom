use redis::{Client, Commands, RedisError};
use serde_json::json;
use twilight_model::id::marker::{GuildMarker, RoleMarker, UserMarker};
use twilight_model::id::Id;
use twilight_model::util::ImageHash;
use serde::{Serialize, Deserialize};
use crate::utils::errors::Error;
use redis::AsyncCommands;

#[derive(Serialize, Deserialize, Debug)]
pub struct PartialGuild {
    pub name: String,
    pub icon: Option<ImageHash>,
    pub roles: Vec<Id<RoleMarker>>
}

#[derive(Clone)]
pub struct RedisConnection {
    client: Client,
}

impl RedisConnection {
    pub fn connect(url: String) -> Result<Self, RedisError> {
        let client = Client::open(url)?;
        Ok(Self { client })
    }

    pub async fn set_guild(&self, id: Id<GuildMarker>, guild: PartialGuild) -> Result<(), RedisError> {
        let mut connection = self.client.get_async_connection().await?;
        let data = json!(guild).to_string();
        connection.set(format!("guilds.{id}"), data).await
    }

    pub async fn get_guild(&self, id: Id<GuildMarker>) -> Result<PartialGuild, Error> {
        let mut connection = self.client.get_async_connection().await.map_err(Error::from)?;
        let data: String = connection.get(format!("guilds.{id}")).await.map_err(Error::from)?;
        serde_json::from_str(data.as_str()).map_err(Error::from)
    }

    pub async fn delete_guild(&self, id: Id<GuildMarker>) -> Result<(), RedisError> {
        let mut connection = self.client.get_async_connection().await?;
        connection.del(format!("guilds.{id}")).await
    }

    pub async fn get_by_position(
        &self,
        path: String,
        position: usize,
    ) -> Result<Option<u32>, RedisError> {
        let mut connection = self.client.get_async_connection().await?;
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
        let mut connection = self.client.get_async_connection().await?;
        let user_id = user_id.to_string();
        let score = connection.zscore(path.clone(), user_id.clone()).await?;
        let position = connection.zrevrank(path, user_id).await?;
        Ok((score, position))
    }

    pub async fn get_all(&self, path: String, limit: isize) -> Result<Vec<(String, u32)>, RedisError> {
        let mut connection = self.client.get_async_connection().await?;
        connection.zrevrange_withscores(path, 0, limit - 1).await
    }

    pub async fn increase(
        &self,
        path: String,
        user_id: Id<UserMarker>,
        count: u8,
    ) -> Result<(), RedisError> {
        let mut connection = self.client.get_async_connection().await?;
        connection.zincr(path, user_id.to_string(), count).await
    }

    #[cfg(feature = "api")]
    pub async fn check_guild(&self, id: Id<GuildMarker>) -> Result<bool, RedisError> {
        let count: u8 = self.client.get_async_connection()
            .await?.exists(format!("guilds.{id}")).await?;
        Ok(count == 1)
    }
}