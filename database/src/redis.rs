use redis::{Client, Commands, RedisError};
use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;

pub struct RedisConnection {
    client: Client
}

impl RedisConnection {

    pub fn connect(url: String) -> Result<Self, RedisError> {
        let client = Client::open(url)?;
        Ok(Self {
            client
        })
    }

    pub fn get_by_position(&self, path: String, position: usize) -> Result<Option<u32>, RedisError> {
        let mut connection = self.client.get_connection()?;
        let result: Vec<u32> = connection.zrevrange_withscores(path, (position - 1) as isize, (position - 1) as isize)?
        Ok(result.first().cloned())
    }

    pub fn get_by_user(&self, path: String, user_id: Id<UserMarker>) -> Result<(u32, u32), RedisError> {
        let mut connection = self.client.get_connection()?;
        let user_id = user_id.to_string();
        let score = connection.zscore(path.clone(), user_id.clone())?;
        let position = connection.zrevrank(path, user_id)?;
        Ok((score, position))
    }

    pub fn get_all(&self, path: String, limit: isize) -> Result<Vec<(String, u32)>, RedisError> {
        let mut connection = self.client.get_connection()?;
        Ok(connection.zrevrange_withscores(path, 0, limit - 1)?)
    }

}

impl Clone for RedisConnection {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone()
        }
    }
}