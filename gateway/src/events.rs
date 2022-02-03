use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use std::sync::Arc;
use twilight_http::Client;
use twilight_model::gateway::event::Event;

pub async fn on_event(
    event: Event,
    mongodb: MongoDBConnection,
    redis: RedisConnection,
    discord_http: Arc<Client>,
) -> Result<(), ()> {
    match event {
        Event::MessageCreate(message) => {
            crate::modules::top::run(message, mongodb, redis, discord_http).await?;
        }
        _ => return Err(()),
    };
    Ok(())
}
