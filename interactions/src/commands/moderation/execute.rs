use std::str::FromStr;
use std::sync::Arc;
use chrono::Utc;
use humantime::Duration;
use mongodb::bson::DateTime;
use twilight_http::Client;
use twilight_http::error::ErrorType;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::channel::message::MessageFlags;
use twilight_model::util::datetime::Timestamp;
use twilight_model::guild::{Member, PartialMember};
use twilight_model::http::interaction::{InteractionResponseData, InteractionResponseType};
use twilight_model::id::Id;
use twilight_model::id::marker::{GenericMarker, GuildMarker, RoleMarker, UserMarker};
use database::models::case::{Case, CaseActionType};
use database::models::config::GuildConfig;
use database::models::config::moderation::MuteMode;
use database::models::task::{Task, TaskAction};
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use utils::check_type;
use utils::constants::duration::{DAY, MINUTE};
use utils::errors::Error;
use utils::modals::{ModalBuilder, RepetitiveTextInput};
use utils::uppercase::FirstLetterToUpperCase;
use crate::commands::ResponseData;
use crate::InteractionContext;

pub async fn run(
    interaction: InteractionContext,
    mongodb: MongoDBConnection,
    redis: RedisConnection,
    discord_http: Arc<Client>,
    config: GuildConfig
) -> ResponseData {
    if let Some(target_user) = interaction.target_id {
        let response = create_modal(
            interaction.command_text, target_user
        ).to_interaction_response_data();
        return Ok((response, Some(InteractionResponseType::Modal)))
    }

    let guild_id = interaction.guild_id.ok_or("This command is guild only")?;
    let member = interaction.member.ok_or("This command is guild only")?;

    let target_id = *check_type!(
        interaction.options.get("member").ok_or("There is no member id")?,
        CommandOptionValue::User
    ).ok_or("Member id type not match")?;

    let reason = get_command_string_option(
        interaction.options.get("reason")
    ).cloned();

    let case_type = command_to_action_type(
        interaction.command_text.as_str(), &config
    ).ok_or("Cannot find action type matching command name")?;

    let target_member = get_target_member(
        &discord_http, guild_id, target_id
    ).await.map_err(Error::from)?;

    if let Some(target_member) = &target_member {
        if !check_position(&redis, guild_id, target_member, member)? {
            return Err(
                Error::from("Missing Permissions: Cannot execute moderation action on user with higher role")
            )
        }
    }

    let duration = get_command_string_option(interaction.options.get("duration"));
    let duration = match duration {
        Some(duration) => {
            let duration = Duration::from_str(duration.as_str())
                .map_err(|_| "Invalid duration string (try 3m, 10s, 2d)")?;
            Some(duration)
        }
        None => None
    };

    if [CaseActionType::Mute, CaseActionType::Timeout].contains(&case_type) {
        let duration = duration.ok_or("Duration is required to mute user")?;
        let end_at = Utc::now().timestamp() + (duration.as_secs() as i64);
        let timestamp = Timestamp::from_secs(end_at).ok();

        if case_type == CaseActionType::Timeout {
            discord_http
                .update_guild_member(guild_id, target_id)
                .communication_disabled_until(timestamp)
                .map_err(Error::from)?
                .exec().await.map_err(Error::from)?
                .model().await.map_err(Error::from)?;
        } else {
            if !verify_mute_duration(duration) {
                return Err(Error::from("Mutes in the role mode must be for min `1m` and max `90d`"))
            }

            let mut roles = target_member
                .ok_or("You can mute only user server members (User left or didn't join this server)")?
                .roles;
            roles.push(config.moderation.mute_role.ok_or("There is no role for muted users set")?);

            discord_http.update_guild_member(config.guild_id, target_id)
                .roles(&roles).exec().await.map_err(Error::from)?;

            mongodb.create_task(Task {
                execute_at: DateTime::from_millis(end_at * 1000),
                guild_id,
                action: TaskAction::RemoveMuteRole(target_id)
            }).await?;
        }
    };

    let index = mongodb.get_next_case_index(guild_id).await? as u16;

    let case = Case {
        moderator_id: interaction.user.id,
        created_at: DateTime::now(),
        guild_id,
        member_id: target_id,
        action: case_type,
        reason,
        removed: false,
        duration: duration.map(|d| d.as_secs() as i64),
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

/// Return true when the duration is correct
fn verify_mute_duration(duration: Duration) -> bool {
    let duration_millis = duration.as_millis() as usize;
    duration_millis > MINUTE && duration_millis < DAY * 90
}

fn command_to_action_type(command_name: &str, config: &GuildConfig) -> Option<CaseActionType> {
    let action_type = match command_name {
        "warn" => CaseActionType::Warn,
        "timeout" | "mute" => {
            match config.moderation.mute_mode {
                MuteMode::Timeout => CaseActionType::Timeout,
                MuteMode::Role => CaseActionType::Mute,
                MuteMode::DependOnCommand => {
                    if command_name == "mute" { CaseActionType::Mute }
                    else { CaseActionType::Timeout }
                }
            }
        },
        "kick" => CaseActionType::Kick,
        "ban" => CaseActionType::Ban,
        _ => return None
    };

    Some(action_type)
}

fn create_modal(command_name: String, target_id: Id<GenericMarker>) -> ModalBuilder {
    let modal = ModalBuilder::new(
        format!("a:{}:{target_id}", command_name),
        command_name.to_owned().first_to_uppercase()
    );

    let modal = if ["mute", "timeout"].contains(&&*command_name) {
        modal.add_repetitive_component(RepetitiveTextInput::Duration)
    } else { modal };

    modal.add_repetitive_component(RepetitiveTextInput::Reason)
}

/// Fetch the guild member, but when the response status is 404 it return `Result::Ok(Option::None)`
async fn get_target_member(
    discord_http: &Arc<Client>,
    guild_id: Id<GuildMarker>,
    member_id: Id<UserMarker>
) -> Result<Option<Member>, Error> {
    match discord_http.guild_member(guild_id, member_id).exec().await {
        Ok(value) => {
            Ok(Some(
                value.model().await.map_err(Error::from)?
            ))
        }
        Err(err) => {
            match err.kind() {
                ErrorType::Response { status, .. } => {
                    if status == &404 { Ok(None) } else { Err(Error::from(err)) }
                },
                _ => Err(Error::from(err))
            }
        }
    }
}

/// Get the highest role from array by checking positions in the sorted array of guild roles
fn get_highest_role_pos(
    sorted_roles: &[Id<RoleMarker>],
    target_roles: &[Id<RoleMarker>]
) -> usize {
    let mut target_role_index = 0;
    for role in target_roles {
        let position = sorted_roles.iter()
            .position(|pos_role| pos_role == role)
            .unwrap_or(0);
        if target_role_index < position { target_role_index = position }
    }
    target_role_index
}

/// Checks is position of the moderator role higher then position of the target role
fn check_position(
    redis: &RedisConnection,
    guild_id: Id<GuildMarker>,
    target_member: &Member,
    member: PartialMember
) -> Result<bool, Error> {
    let guild = redis.get_guild(guild_id).map_err(Error::from)?;

    let target_role_index = get_highest_role_pos(
        &guild.roles,
        &target_member.roles
    );

    let moderator_role_index = get_highest_role_pos(
        &guild.roles,
        &member.roles
    );

    Ok(target_role_index < moderator_role_index)
}

fn get_command_string_option(value: Option<&CommandOptionValue>) -> Option<&String> {
    match value {
        Some(CommandOptionValue::String(value)) => {
            if value.is_empty() { None } else { Some(value) }
        },
        Some(_) => None,
        None => None
    }
}