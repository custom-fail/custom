use std::sync::Arc;
use twilight_http::Client;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::http::interaction::{InteractionResponseData, InteractionResponseType};
use database::models::case::Case;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use mongodb::bson::DateTime;
use twilight_model::channel::message::MessageFlags;
use database::models::config::GuildConfig;
use utils::check_type;
use utils::modals::{ModalBuilder, RepetitiveTextInput};
use crate::commands::ResponseData;
use crate::InteractionContext;

pub async fn run(interaction: InteractionContext, mongodb: MongoDBConnection, _: RedisConnection, discord_http: Arc<Client>, config: GuildConfig) -> ResponseData {

    if let Some(target_user) = interaction.target_id {
        return Ok((
            ModalBuilder::new(format!("a:warn:{target_user}"), "Warn".to_string())
                .add_repetitive_component(RepetitiveTextInput::Reason)
                .to_interaction_response_data(),
            Some(InteractionResponseType::Modal)
        ))
    }

    let user_id = interaction.user.ok_or("Cannot find executor")?.id;
    let guild_id = interaction.guild_id.ok_or("This is guild only")?;

    let member_id = check_type!(
        interaction.options.get("member").ok_or("There is no member id")?,
        CommandOptionValue::User
    ).ok_or("Member id type not match")?.clone();

    let reason = match interaction.options.get("reason") {
        Some(CommandOptionValue::String(value)) => Some(value),
        Some(_) => None,
        None => None
    }.cloned();

    let index = mongodb.get_next_case_index(guild_id).await? as u16;

    let case = Case {
        moderator_id: user_id,
        created_at: DateTime::now(),
        guild_id,
        member_id,
        action: 1,
        reason,
        removed: false,
        duration: None,
        index
    };

    let case_embed = case.to_embed(discord_http.clone()).await?;

    let result = mongodb.create_case(
        discord_http, case, case_embed.clone(),
        if config.moderation.dm_case { Some(member_id) } else { None },
        config.moderation.logs_channel
    ).await.map_err(|err| format!("{:?}", err));

    Ok((InteractionResponseData {
        allowed_mentions: None,
        attachments: None,
        choices: None,
        components: None,
        content: check_type!(result, Err),
        custom_id: None,
        embeds: Some(vec![case_embed]),
        flags: Some(MessageFlags::EPHEMERAL),
        title: None,
        tts: None
    }, None))

}