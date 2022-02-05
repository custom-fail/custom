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
        Event::MessageCreate(event) => {
            crate::modules::top::run(event, mongodb, redis, discord_http).await;
        }
        Event::BanAdd(event) => { crate::modules::case::on_ban::run(event, mongodb, discord_http).await; },
        Event::MemberRemove(event) => { crate::modules::case::on_kick::run(event, mongodb, discord_http).await; },
        Event::MemberUpdate(event) => {
            crate::modules::case::on_timeout::run(event, mongodb, discord_http).await;
        }
        _ => return Err(()),
    };
    Ok(())
}
