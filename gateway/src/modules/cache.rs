use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use twilight_model::util::ImageHash;
use database::redis::{PartialGuild, RedisConnection};

pub fn set_guild(redis: RedisConnection, id: Id<GuildMarker>, name: String, icon: Option<ImageHash>) -> Result<(), ()> {
    redis.set_guild(id, PartialGuild { name, icon }).map_err(|_| ())
}

pub fn delete_guild(redis: RedisConnection, id: Id<GuildMarker>) -> Result<(), ()> {
    redis.delete_guild(id).map_err(|_| ())
}