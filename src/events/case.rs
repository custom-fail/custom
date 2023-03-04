use std::sync::Arc;
use mongodb::bson::DateTime;
use twilight_http::Client;
use twilight_model::gateway::payload::incoming::GuildAuditLogEntryCreate;
use twilight_model::guild::audit_log::{AuditLogChange, AuditLogEventType};
use twilight_util::snowflake::Snowflake;
use crate::context::Context;
use crate::models::case::{Case, CaseActionType};

pub async fn run(event: Box<GuildAuditLogEntryCreate>, discord_http: Arc<Client>, context: Arc<Context>) -> Result<(), ()> {
    let action_type = match event.action_type {
        AuditLogEventType::MemberKick => CaseActionType::Kick,
        AuditLogEventType::MemberBanAdd => CaseActionType::Ban,
        AuditLogEventType::MemberUpdate => CaseActionType::Timeout,
        _ => return Ok(())
    };

    let moderator_id = event.user_id.ok_or(())?;
    let guild_id = event.guild_id.ok_or(())?;
    let target_id = event.target_id.ok_or(())?;

    let guild_config = context.mongodb.get_config(guild_id).await.map_err(|_| ())?;
    if !guild_config.moderation.native_support {
        return Err(())
    }

    let change = event.changes.last().ok_or(())?;
    let created_at = event.id.timestamp();

    let duration = if let AuditLogChange::CommunicationDisabledUntil { old, new } = change {
        if old.is_some() { return Err(()) };
        if let Some(ends_on) = new {
            Some(ends_on.as_secs() - created_at / 1000)
        } else {
            return Err(())
        }
    } else if event.action_type == AuditLogEventType::MemberUpdate {
        return Ok(()) // its not a timeout
    } else { None };

    let moderator = discord_http.user(moderator_id).await.map_err(|_| ())?.model().await.map_err(|_| ())?;
    if moderator.bot { return Ok(()) }

    let count = context.mongodb.get_next_case_index(guild_id).await.map_err(|_| ())?;

    let case = Case {
        moderator_id,
        created_at: DateTime::from_millis(created_at),
        guild_id,
        member_id: target_id.cast(),
        action: action_type,
        reason: event.reason.to_owned(),
        removed: false,
        duration,
        index: count as u16
    };

    let embed = case.to_embed(discord_http.clone()).await.map_err(|_| ())?;

    context.mongodb.create_case(
        discord_http.clone(),
        &context.redis,
        case,
        embed,
        if guild_config.moderation.dm_case { Some(target_id.cast()) } else { None },
        guild_config.moderation.logs_channel
    ).await.map_err(|_| ())?;

    Ok(())
}