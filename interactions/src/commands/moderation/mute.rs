use std::str::FromStr;
use std::sync::Arc;
use chrono::Utc;
use twilight_http::Client;
use twilight_model::http::interaction::{InteractionResponseData, InteractionResponseType};
use database::models::config::GuildConfig;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use humantime::Duration;
use mongodb::bson::DateTime;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::channel::message::MessageFlags;
use twilight_model::datetime::Timestamp;
use database::models::case::Case;
use utils::check_type;
use utils::modals::{ModalBuilder, RepetitiveTextInput};
use crate::commands::ResponseData;
use crate::InteractionContext;

pub async fn run(interaction: InteractionContext, mongodb: MongoDBConnection, _: RedisConnection, discord_http: Arc<Client>, config: GuildConfig) -> ResponseData {

    if let Some(target_user) = interaction.target_id {
        return Ok((
            ModalBuilder::new(format!("a:mute:{target_user}"), "Mute".to_string())
                .add_repetitive_component(RepetitiveTextInput::Duration)
                .add_repetitive_component(RepetitiveTextInput::Reason)
                .to_interaction_response_data(),
             Some(InteractionResponseType::Modal)
        ));
    }

    let user_id = interaction.user.ok_or("Cannot find executor".to_string())?.id;
    let guild_id = interaction.guild_id.ok_or("This is guild only".to_string())?;

    let member_id = check_type!(
        interaction.options.get("member").ok_or("There is no member id".to_string())?,
        CommandOptionValue::User
    ).ok_or("Member id type not match".to_string())?.clone();

    let reason = match interaction.options.get("reason") {
        Some(CommandOptionValue::String(value)) => Some(value),
        Some(_) => None,
        None => None
    }.cloned();

    let duration = check_type!(
        interaction.options.get("duration").ok_or("There is no reason".to_string())?,
       CommandOptionValue::String
    ).ok_or("Duration type not match".to_string())?.clone();

    let duration = Duration::from_str(duration.as_str()).map_err(|_| "Invalid duration string (try 3m, 10s, 2d)".to_string())?;
    let end_at = Utc::now().timestamp() + (duration.as_secs() as i64);

    let timestamp = Timestamp::from_secs(end_at).ok();

    discord_http
        .update_guild_member(guild_id, member_id)
        .communication_disabled_until(timestamp)
        .map_err(|err| err.to_string())?
        .exec().await.map_err(|err| err.to_string())?
        .model().await.map_err(|err| err.to_string())?;

    let index = mongodb.get_next_case_index(guild_id).await? as u16;

    let case = Case {
        moderator_id: user_id,
        created_at: DateTime::now(),
        guild_id,
        member_id,
        action: 7,
        reason,
        removed: false,
        duration: Some(duration.as_secs() as i64),
        index
    };

    let case_embed = case.to_embed(discord_http.clone()).await?;

    mongodb.create_case(
        discord_http.clone(), case, case_embed.clone(),
        if config.moderation.dm_case { Some(member_id) } else { None },
        config.moderation.logs_channel
    ).await.ok();

    Ok((InteractionResponseData {
        allowed_mentions: None,
        attachments: None,
        choices: None,
        components: None,
        content: None,
        custom_id: None,
        embeds: Some(vec![case_embed]),
        flags: Some(MessageFlags::EPHEMERAL),
        title: None,
        tts: None
    }, None))
}