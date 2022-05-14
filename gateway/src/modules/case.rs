use std::sync::Arc;
use chrono::Utc;
use mongodb::bson::DateTime;
use twilight_http::Client;
use twilight_model::guild::audit_log::{AuditLogChange, AuditLogEventType};
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_util::snowflake::Snowflake;
use database::models::case::Case;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;

pub async fn run(
    mongodb: MongoDBConnection,
    discord_http: Arc<Client>,
    redis: RedisConnection,
    guild_id: Id<GuildMarker>,
    target_id: Id<UserMarker>,
    action_type: (AuditLogEventType, u8)
) -> Result<(), ()> {

    let event_at = Utc::now().timestamp();

    let guild_config = mongodb.get_config(guild_id).await.map_err(|_| ())?;
    if !guild_config.moderation.native_support {
        return Err(())
    }

    let audit_log = discord_http
        .audit_log(guild_id)
        .action_type(action_type.0)
        .limit(1).map_err(|_| ())?
        .exec().await.map_err(|_| ())?.model().await.map_err(|_| ())?;

    let action = audit_log.entries.first().ok_or(())?;
    let action_target_id = action.target_id.ok_or(())?;
    if action_target_id.to_string() != target_id.to_string() {
        return Err(());
    };

    let created_at = action.id.timestamp();
    let ping = created_at / 1000 - event_at;
    if ping > 2 {
        return Err(());
    }

    let duration = if action_type.1 == 7 {
        let change = action.changes.last().ok_or(())?;

        if let AuditLogChange::CommunicationDisabledUntil { old: _, new } = change {
            let ends_on = new.ok_or(())?;
            Some(ends_on.as_secs() - created_at / 1000)
        } else {
            return Err(())
        }
    } else { None };

    let moderator = action.user_id.ok_or(())?;

    let user = audit_log.users.iter().find(|u| u.id == moderator).ok_or(())?;
    if user.bot { return Ok(()) }

    let count = mongodb.get_next_case_index(guild_id).await.map_err(|_| ())?;

    let case = Case {
        moderator_id: moderator,
        created_at: DateTime::now(),
        guild_id,
        member_id: target_id,
        action: action_type.1,
        reason: action.reason.clone(),
        removed: false,
        duration,
        index: count as u16
    };

    let embed = case.to_embed(discord_http.clone()).await.map_err(|_| ())?;

    mongodb.create_case(
        discord_http.clone(),
        redis,
        case,
        embed,
        if guild_config.moderation.dm_case { Some(target_id) } else { None },
        guild_config.moderation.logs_channel
    ).await.map_err(|_| ())?;

    Ok(())

}

pub mod on_kick {
    use std::sync::Arc;
    use twilight_http::Client;
    use twilight_model::gateway::payload::incoming::MemberRemove;
    use twilight_model::guild::audit_log::AuditLogEventType;
    use database::mongodb::MongoDBConnection;
    use database::redis::RedisConnection;

    pub async fn run(
        event: MemberRemove,
        mongodb: MongoDBConnection,
        discord_http: Arc<Client>,
        redis: RedisConnection
    ) -> Result<(), ()> {
        crate::modules::case::run(mongodb, discord_http, redis, event.guild_id, event.user.id, (AuditLogEventType::MemberKick, 6)).await
    }
}

pub mod on_ban {
    use database::mongodb::MongoDBConnection;
    use std::sync::Arc;
    use twilight_http::Client;
    use twilight_model::gateway::payload::incoming::BanAdd;
    use twilight_model::guild::audit_log::AuditLogEventType;
    use database::redis::RedisConnection;

    pub async fn run(
        event: BanAdd,
        mongodb: MongoDBConnection,
        discord_http: Arc<Client>,
        redis: RedisConnection
    ) -> Result<(), ()> {
        crate::modules::case::run(mongodb, discord_http, redis, event.guild_id, event.user.id, (AuditLogEventType::MemberBanAdd, 4)).await
    }
}

pub mod on_timeout {
    use database::mongodb::MongoDBConnection;
    use std::sync::Arc;
    use twilight_http::Client;
    use twilight_model::gateway::payload::incoming::MemberUpdate;
    use twilight_model::guild::audit_log::AuditLogEventType;
    use database::redis::RedisConnection;

    pub async fn run(
        event: Box<MemberUpdate>,
        mongodb: MongoDBConnection,
        discord_http: Arc<Client>,
        redis: RedisConnection
    ) -> Result<(), ()> {
        crate::modules::case::run(mongodb, discord_http, redis, event.guild_id, event.user.id, (AuditLogEventType::MemberUpdate, 7)).await
    }
}
