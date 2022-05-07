use twilight_model::channel::Message;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;

pub async fn run(
    message: Message,
    mongodb: MongoDBConnection,
    redis: RedisConnection
) -> Result<(), ()> {
    if message.author.bot {
        return Err(());
    }

    let guild_id = message.guild_id.ok_or(())?;
    let config = mongodb.get_config(guild_id).await.map_err(|_| ())?;
    let author_id = message.author.id;

    if config.top.week {
        redis
            .increase(format!("top_week.{guild_id}"), author_id, 1)
            .map_err(|_| ())?;
    }

    if config.top.day {
        redis
            .increase(format!("top_day.{guild_id}"), author_id, 1)
            .map_err(|_| ())?;
    }

    Ok(())
}
