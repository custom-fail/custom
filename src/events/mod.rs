use std::sync::Arc;
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use crate::{Bucket, MongoDBConnection, RedisConnection, ScamLinks};

pub mod automod;
pub mod case;
pub mod top;
pub mod cache;
pub mod restore;
pub mod setup;

pub async fn on_event(
    event: Event,
    mongodb: MongoDBConnection,
    redis: RedisConnection,
    discord_http: Arc<Client>,
    scam_domains: ScamLinks,
    bucket: Bucket
) -> Result<(), ()> {
    match event {
        Event::MemberAdd(event) => {
            self::restore::mutes::run(event, mongodb, discord_http).await.ok();
        }
        Event::BanRemove(event) => {
            self::restore::bans::run(event, mongodb).await.ok();
        }
        Event::MessageCreate(event) => {
            let message = event.as_ref().0.to_owned();
            self::automod::run(message.clone(), mongodb.clone(), discord_http, scam_domains, bucket).await.ok();
            self::top::run(message, mongodb, redis).await.ok();
        }
        Event::BanAdd(event) => { self::case::on_ban::run(event, mongodb, discord_http, redis).await.ok(); },
        Event::MemberRemove(event) => { self::case::on_kick::run(event, mongodb, discord_http, redis).await.ok(); },
        Event::MemberUpdate(event) => {
            self::case::on_timeout::run(event, mongodb, discord_http, redis).await.ok();
        },
        Event::GuildCreate(event) => {
            tokio::spawn(self::setup::run(event.id, event.joined_at, discord_http));
            self::cache::on_guild_create(redis, event).ok();
        },
        Event::GuildUpdate(event) => {
            self::cache::on_guild_update(redis, event).ok();
        },
        Event::GuildDelete(event) => {
            self::cache::delete_guild(redis, event.id).ok();
        },
        Event::RoleCreate(event) => {
            self::cache::fetch_and_set(redis, discord_http, event.guild_id).await.ok();
        },
        Event::RoleUpdate(event) => {
            self::cache::fetch_and_set(redis, discord_http, event.guild_id).await.ok();
        },
        Event::RoleDelete(event) => {
            self::cache::fetch_and_set(redis, discord_http, event.guild_id).await.ok();
        }
        _ => return Err(()),
    };
    Ok(())
}
