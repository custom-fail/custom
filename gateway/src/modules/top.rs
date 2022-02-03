use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use std::sync::Arc;
use twilight_http::Client;
use twilight_model::gateway::payload::incoming::MessageCreate;

pub async fn run(
    message: Box<MessageCreate>,
    mongodb: MongoDBConnection,
    redis: RedisConnection,
    _: Arc<Client>,
) -> Result<(), ()> {
    let guild_id = message.guild_id.ok_or(())?;
    let config = mongodb.get_config(guild_id).await.map_err(|_| ())?;
    let author_id = message.author.id;

    if config.top.week {
        redis
            .increase(format!("top_week.{}", guild_id), author_id, 1)
            .map_err(|_| ())?;
    }

    if config.top.day {
        redis
            .increase(format!("top_day.{}", guild_id), author_id, 1)
            .map_err(|_| ())?;
    }

    Ok(())
}
