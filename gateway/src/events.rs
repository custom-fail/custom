use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use std::sync::Arc;
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use crate::ScamLinks;

pub async fn on_event(
    event: Event,
    mongodb: MongoDBConnection,
    redis: RedisConnection,
    discord_http: Arc<Client>,
    scam_domains: ScamLinks
) -> Result<(), ()> {
    match event {
        Event::MessageCreate(event) => {
            crate::modules::auto::run(event.clone(), mongodb.clone(), discord_http, scam_domains).await.ok();
            crate::modules::top::run(event, mongodb, redis).await.ok();
        }
        Event::BanAdd(event) => { crate::modules::case::on_ban::run(event, mongodb, discord_http).await.ok(); },
        Event::MemberRemove(event) => { crate::modules::case::on_kick::run(event, mongodb, discord_http).await.ok(); },
        Event::MemberUpdate(event) => {
            crate::modules::case::on_timeout::run(event, mongodb, discord_http).await.ok();
        }
        _ => return Err(()),
    };
    Ok(())
}
