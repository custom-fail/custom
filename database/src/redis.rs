use redis::{Client, RedisError};

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

}