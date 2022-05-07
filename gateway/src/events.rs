use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use std::sync::Arc;
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use crate::{Bucket, ScamLinks};

pub async fn on_event(
    event: Event,
    mongodb: MongoDBConnection,
    redis: RedisConnection,
    discord_http: Arc<Client>,
    scam_domains: ScamLinks,
    bucket: Bucket
) -> Result<(), ()> {
    match event {
        Event::MessageCreate(event) => {
            let message = event.as_ref().0.to_owned();
            crate::modules::automod::run(message.clone(), mongodb.clone(), discord_http, scam_domains, bucket).await.ok();
            crate::modules::top::run(message, mongodb, redis).await.ok();
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
