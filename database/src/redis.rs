use redis::{Client, Commands, RedisError};
use twilight_model::id::Id;
use twilight_model::user::User;

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

    pub fn get_by_user(&self, path: String, user_id: Id<User>) -> Result<(u32, u32), RedisError> {
        let mut connection = self.client.get_connection()?;
        let user_id = user_id.to_string();
        let score = connection.zscore(path.clone(), user_id.clone())?;
        let position = connection.zrevrank(path, user_id)?;
        Ok((score, position))
    }

}