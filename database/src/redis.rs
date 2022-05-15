use redis::{Client, Commands, RedisError};
use serde_json::json;
use twilight_model::id::marker::{GuildMarker, RoleMarker, UserMarker};
use twilight_model::id::Id;
use twilight_model::util::ImageHash;
use serde::{Serialize, Deserialize};
use utils::errors::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct PartialGuild {
    pub name: String,
    pub icon: Option<ImageHash>,
    pub roles: Vec<Id<RoleMarker>>
}

pub struct RedisConnection {
    client: Client,
}

impl RedisConnection {
    pub fn connect(url: String) -> Result<Self, RedisError> {
        let client = Client::open(url)?;
        Ok(Self { client })
    }

    pub fn set_guild(&self, id: Id<GuildMarker>, guild: PartialGuild) -> Result<(), RedisError> {
        let mut connection = self.client.get_connection()?;
        let data = json!(guild).to_string();
        connection.set(format!("guilds.{id}"), data)
    }

    pub fn get_guild(&self, id: Id<GuildMarker>) -> Result<PartialGuild, Error> {
        let mut connection = self.client.get_connection().map_err(Error::from)?;
        let data: String = connection.get(format!("guilds.{id}")).map_err(Error::from)?;
        serde_json::from_str(data.as_str()).map_err(Error::from)
    }

    pub fn delete_guild(&self, id: Id<GuildMarker>) -> Result<(), RedisError> {
        let mut connection = self.client.get_connection()?;
        connection.del(format!("guilds.{id}"))
    }

    pub fn get_by_position(
        &self,
        path: String,
        position: usize,
    ) -> Result<Option<u32>, RedisError> {
        let mut connection = self.client.get_connection()?;
        let result: Vec<u32> = connection.zrevrange_withscores(
            path,
            (position - 1) as isize,
            (position - 1) as isize,
        )?;
        Ok(result.first().cloned())
    }

    pub fn get_by_user(
        &self,
        path: String,
        user_id: Id<UserMarker>,
    ) -> Result<(u32, u32), RedisError> {
        let mut connection = self.client.get_connection()?;
        let user_id = user_id.to_string();
        let score = connection.zscore(path.clone(), user_id.clone())?;
        let position = connection.zrevrank(path, user_id)?;
        Ok((score, position))
    }

    pub fn get_all(&self, path: String, limit: isize) -> Result<Vec<(String, u32)>, RedisError> {
        let mut connection = self.client.get_connection()?;
        connection.zrevrange_withscores(path, 0, limit - 1)
    }

    pub fn increase(
        &self,
        path: String,
        user_id: Id<UserMarker>,
        count: u8,
    ) -> Result<(), RedisError> {
        let mut connection = self.client.get_connection()?;
        connection.zincr(path, user_id.to_string(), count)
    }
}

impl Clone for RedisConnection {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}
