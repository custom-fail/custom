use std::sync::Arc;
use twilight_http::Client;
use twilight_model::gateway::payload::incoming::{GuildCreate, GuildUpdate};
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use database::redis::{PartialGuild, RedisConnection};

pub async fn fetch_and_set(
    redis: RedisConnection,
    discord_http: Arc<Client>,
    guild_id: Id<GuildMarker>
) -> Result<(), ()> {
    let mut guild = discord_http.guild(guild_id)
        .exec().await.map_err(|_| ())?
        .model().await.map_err(|_| ())?;
    guild.roles.sort_by_cached_key(|role| role.position);
    set_guild(redis, guild_id, PartialGuild {
        name: guild.name,
        icon: guild.icon,
        roles: guild.roles.iter().map(|role| role.id).collect()
    })
}

pub fn on_guild_create(redis: RedisConnection, event: Box<GuildCreate>) -> Result<(), ()> {
    let mut roles = event.roles.to_owned();
    roles.sort_by_cached_key(|role| role.position);
    set_guild(redis, event.id, PartialGuild {
        name: event.name.to_owned(),
        icon: event.icon,
        roles: roles.iter().map(|role| role.id).collect()
    })
}

pub fn on_guild_update(redis: RedisConnection, event: Box<GuildUpdate>) -> Result<(), ()> {
    let mut roles = event.roles.to_owned();
    roles.sort_by_cached_key(|role| role.position);
    set_guild(redis, event.id, PartialGuild {
        name: event.name.to_owned(),
        icon: event.icon,
        roles: roles.iter().map(|role| role.id).collect()
    })
}

pub fn set_guild(redis: RedisConnection, id: Id<GuildMarker>, guild: PartialGuild) -> Result<(), ()> {
    redis.set_guild(id, guild).map_err(|_| ())
}

pub fn delete_guild(redis: RedisConnection, id: Id<GuildMarker>) -> Result<(), ()> {
    redis.delete_guild(id).map_err(|_| ())
}