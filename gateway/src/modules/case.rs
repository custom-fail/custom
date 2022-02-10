use std::sync::Arc;
use chrono::Utc;
use mongodb::bson::{DateTime, doc};
use twilight_http::Client;
use twilight_model::guild::audit_log::AuditLogEventType;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_util::snowflake::Snowflake;
use database::models::case::Case;
use database::mongodb::MongoDBConnection;

pub async fn run(mongodb: MongoDBConnection, discord_http: Arc<Client>, guild_id: Id<GuildMarker>, target_id: Id<UserMarker>, action_type: (AuditLogEventType, u8)) -> Result<(), ()> {

    let event_at = Utc::now().timestamp();

    let guild_config = mongodb.get_config(guild_id.clone()).await.map_err(|_| ())?;
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

    let moderator = action.user_id.ok_or(())?.clone();

    let created_at = action.id.timestamp();
    let ping = created_at - event_at * 1000;

    if ping > 2000 {
        return Err(());
    }

    let count = mongodb.cases.count_documents(doc! {}, None).await.map_err(|_| ())?;

    let ok = mongodb.cases.insert_one(Case {
        moderator_id: moderator,
        created_at: DateTime::now(),
        guild_id,
        member_id: target_id,
        action: action_type.1,
        reason: action.reason.clone(),
        removed: false,
        duration: None,
        index: (count + 1) as u16
    }, None).await;

    Ok(())

}

pub mod on_kick {
    use std::sync::Arc;
    use twilight_http::Client;
    use twilight_model::gateway::payload::incoming::MemberRemove;
    use twilight_model::guild::audit_log::AuditLogEventType;
    use database::mongodb::MongoDBConnection;

    pub async fn run(
        event: MemberRemove,
        mongodb: MongoDBConnection,
        discord_http: Arc<Client>,
    ) -> Result<(), ()> {

        crate::modules::case::run(mongodb, discord_http, event.guild_id, event.user.id, (AuditLogEventType::MemberKick, 6)).await

    }
}

pub mod on_ban {
    use database::mongodb::MongoDBConnection;
    use std::sync::Arc;
    use twilight_http::Client;
    use twilight_model::gateway::payload::incoming::BanAdd;
    use twilight_model::guild::audit_log::AuditLogEventType;

    pub async fn run(event: BanAdd, mongodb: MongoDBConnection, discord_http: Arc<Client>) -> Result<(), ()> {
        crate::modules::case::run(mongodb, discord_http, event.guild_id, event.user.id, (AuditLogEventType::MemberBanAdd, 4)).await
    }
}

pub mod on_timeout {
    use database::mongodb::MongoDBConnection;
    use std::sync::Arc;
    use twilight_http::Client;
    use twilight_model::gateway::payload::incoming::MemberUpdate;
    use twilight_model::guild::audit_log::AuditLogEventType;

    pub async fn run(event: Box<MemberUpdate>, mongodb: MongoDBConnection, discord_http: Arc<Client>) -> Result<(), ()> {
        Ok(())
    }
}
