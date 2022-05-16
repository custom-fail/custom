use std::str::FromStr;
use std::sync::Arc;
use chrono::Utc;
use humantime::Duration;
use mongodb::bson::DateTime;
use twilight_http::Client;
use twilight_http::error::ErrorType;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::channel::message::MessageFlags;
use twilight_model::datetime::Timestamp;
use twilight_model::guild::{Member, PartialMember};
use twilight_model::http::interaction::{InteractionResponseData, InteractionResponseType};
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use database::models::case::Case;
use database::models::config::GuildConfig;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use utils::check_type;
use utils::errors::Error;
use utils::modals::{ModalBuilder, RepetitiveTextInput};
use utils::uppercase::FirstLetterToUpperCase;
use crate::commands::ResponseData;
use crate::InteractionContext;

async fn get_target_member(
    discord_http: &Arc<Client>,
    guild_id: Id<GuildMarker>,
    member_id: Id<UserMarker>
) -> Result<Option<Member>, Error> {
    match discord_http.guild_member(guild_id, member_id).exec().await {
        Ok(value) => match value.model().await {
            Ok(member) => Ok(Some(member)),
            Err(err) => return Err(Error::from(err))
        }
        Err(err) => {
            return match err.kind() {
                ErrorType::Response { status, .. } => {
                    if status == &404 { Ok(None) } else { Err(Error::from(err)) }
                },
                _ => Err(Error::from(err))
            }
        }
    }
}

pub fn check_position(
    redis: &RedisConnection,
    guild_id: Id<GuildMarker>,
    target_member: Member,
    member: PartialMember
) -> Result<bool, Error> {

    let moderator_role = match member.roles.first() {
        Some(role) => role,
        None => return Ok(false)
    };

    let guild = redis.get_guild(guild_id).map_err(Error::from)?;
    let everyone_position = guild.roles.len();

    let mut target_role_index = None;
    for role in target_member.roles.to_owned() {
        let position = guild.roles.iter()
            .position(|pos_role| pos_role == &role)
            .unwrap_or(everyone_position);
        if let Some(index) = target_role_index {
            if index < position {
                target_role_index = Some(position);
            }
        } else {
            target_role_index = Some(position);
        }
    }

    let target_role_index = match target_role_index {
        Some(role) => role,
        None => return Ok(true)
    };

    let moderator_role_index = guild.roles.iter()
        .position(|role| role == moderator_role).unwrap_or(everyone_position);

    Ok(target_role_index < moderator_role_index)

}

pub async fn run(
    interaction: InteractionContext,
    mongodb: MongoDBConnection,
    redis: RedisConnection,
    discord_http: Arc<Client>,
    config: GuildConfig
) -> ResponseData {

    if let Some(target_user) = interaction.target_id {
        return if *&interaction.command_text == "mute" {
            Ok((
                ModalBuilder::new(format!("a:mute:{target_user}"), "Mute".to_string())
                    .add_repetitive_component(RepetitiveTextInput::Duration)
                    .add_repetitive_component(RepetitiveTextInput::Reason)
                    .to_interaction_response_data(),
                Some(InteractionResponseType::Modal)
            ))
        } else {
            Ok((
                ModalBuilder::new(
                    format!("a:{}:{target_user}", interaction.command_text),
                    interaction.command_text.first_to_uppercase()
                )
                    .add_repetitive_component(RepetitiveTextInput::Reason)
                    .to_interaction_response_data(),
                Some(InteractionResponseType::Modal)
            ))
        }
    }

    let user_id = interaction.user.ok_or("Cannot find executor")?.id;
    let guild_id = interaction.guild_id.ok_or("This command is guild only")?;
    let member = interaction.member.ok_or("This command is guild only")?;

    let target_id = *check_type!(
        interaction.options.get("member").ok_or("There is no member id")?,
        CommandOptionValue::User
    ).ok_or("Member id type not match")?;

    let reason = match interaction.options.get("reason") {
        Some(CommandOptionValue::String(value)) => {
            if value.is_empty() { None }
            else { Some(value) }
        },
        Some(_) => None,
        None => None
    }.cloned();

    let case_type = match interaction.command_text.as_str() {
        "mute" => 7,
        "warn" => 1,
        "ban" => 4,
        "kick" => 6,
        _ => return Err(Error::from("Invalid action"))
    };

    if target_id != user_id {
        let target_member = get_target_member(
            &discord_http, guild_id, target_id
        ).await.map_err(Error::from)?;

        if let Some(target_member) = target_member {
            if !check_position(&redis, guild_id, target_member, member)? {
                return Err(
                    Error::from("Missing Permissions: Cannot execute moderation action on user with higher role")
                )
            }
        }
    }

    let mut case_duration = None;
    if *&interaction.command_text == "mute" {
        let duration = check_type!(
            interaction.options.get("duration").ok_or("There is no duration")?,
            CommandOptionValue::String
        ).ok_or("Duration type not match")?.to_owned();

        let duration = Duration::from_str(duration.as_str())
            .map_err(|_| "Invalid duration string (try 3m, 10s, 2d)")?;
        let end_at = Utc::now().timestamp() + (duration.as_secs() as i64);

        let timestamp = Timestamp::from_secs(end_at).ok();
        case_duration = Some(duration.as_secs() as i64);

        discord_http
            .update_guild_member(guild_id, target_id)
            .communication_disabled_until(timestamp)
            .map_err(Error::from)?
            .exec().await.map_err(Error::from)?
            .model().await.map_err(Error::from)?;
    }

    let index = mongodb.get_next_case_index(guild_id).await? as u16;

    let case = Case {
        moderator_id: user_id,
        created_at: DateTime::now(),
        guild_id,
        member_id: target_id,
        action: case_type,
        reason,
        removed: false,
        duration: case_duration,
        index
    };

    let result_action = match interaction.command_text.as_str() {
        "kick" => {
            discord_http.remove_guild_member(guild_id, target_id).exec().await.err()
        },
        "ban" => {
            discord_http.create_ban(guild_id, target_id).exec().await.err()
        },
        _ => None
    };

    let case_embed = case.to_embed(discord_http.to_owned()).await?;

    let result_case = mongodb.create_case(
        discord_http.to_owned(), redis, case,
        case_embed.to_owned(),
        if config.moderation.dm_case { Some(target_id) } else { None },
        config.moderation.logs_channel
    ).await.err();

    Ok((InteractionResponseData {
        allowed_mentions: None,
        attachments: None,
        choices: None,
        components: None,
        content: if result_action.is_some() || result_case.is_some() {
            Some(format!("Action status: {result_action:?}\nCase status: {result_case:?}"))
        } else { None },
        custom_id: None,
        embeds: Some(vec![case_embed]),
        flags: Some(MessageFlags::EPHEMERAL),
        title: None,
        tts: None
    }, None))

}