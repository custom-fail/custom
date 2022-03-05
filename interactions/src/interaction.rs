use std::sync::Arc;
use twilight_model::application::interaction::{ApplicationCommand, Interaction, MessageComponentInteraction};
use twilight_model::channel::message::MessageFlags;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use twilight_http::Client;
use twilight_model::application::interaction::modal::ModalSubmitInteraction;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use crate::Application;
use crate::commands::context::InteractionContext;
use crate::commands::{parse_slash_command_to_text, ResponseData};

pub async fn handle_interaction(interaction: Interaction, application: Application, mongodb: MongoDBConnection, redis: RedisConnection, discord_http: Arc<Client>) -> InteractionResponse {
    match interaction {
        Interaction::Ping(_) => InteractionResponse {
            kind: InteractionResponseType::Pong,
            data: None
        },
        _ => {
            let mut response_type_default = InteractionResponseType::ChannelMessageWithSource;
            let response = match interaction {
                Interaction::ApplicationCommand(interaction) => {
                    commands_handler(interaction, application, mongodb, redis, discord_http).await
                }
                Interaction::MessageComponent(interaction) => {
                    response_type_default = InteractionResponseType::UpdateMessage;
                    component_handler(interaction, application, mongodb, redis, discord_http).await
                },
                Interaction::ModalSubmit(interaction) => {
                    modal_handler(interaction, application, mongodb, redis, discord_http).await
                }
                _ => Err("Not supported interaction type".to_string())
            };

            match response {
                Ok((response, response_type)) => InteractionResponse {
                    kind: response_type.unwrap_or(response_type_default),
                    data: Some(response)
                },
                Err(error) => InteractionResponse {
                    kind: InteractionResponseType::ChannelMessageWithSource,
                    data: Some(InteractionResponseData {
                        allowed_mentions: None,
                        attachments: None,
                        choices: None,
                        components: None,
                        content: Some(error),
                        custom_id: None,
                        embeds: None,
                        flags: Some(MessageFlags::EPHEMERAL),
                        title: None,
                        tts: None
                    })
                }
            }
        }
    }
}

async fn component_handler(interaction: Box<MessageComponentInteraction>, application: Application, mongodb: MongoDBConnection, redis: RedisConnection, discord_http: Arc<Client>) -> ResponseData {

    let context = InteractionContext::from_message_component_interaction(interaction, application.clone()).await?;
    let command = application.find_command(context.command_text.clone()).await.ok_or("Cannot find command")?;

    (command.run)(context, mongodb, redis, discord_http).await

}

async fn commands_handler(interaction: Box<ApplicationCommand>, application: Application, mongodb: MongoDBConnection, redis: RedisConnection, discord_http: Arc<Client>) -> ResponseData {

    let command_vec = parse_slash_command_to_text(interaction.data.clone());
    let command_text = command_vec.clone().join(" ");
    let command = application.find_command(command_text.clone()).await.ok_or("Cannot find command")?;

    let guild_id = interaction.guild_id.ok_or("Cannot find guild_id".to_string())?;
    let config = mongodb.get_config(guild_id).await.map_err(|_| "Cannot find guild config".to_string())?;

    let context = InteractionContext::from_command_data(interaction.clone(), (command_vec.clone(), command_text.clone()));

    config.enabled.get(command.module.as_str()).ok_or("This module is disabled".to_string())?;

    (command.run)(context, mongodb, redis, discord_http).await

}

async fn modal_handler(interaction: Box<ModalSubmitInteraction>, application: Application, mongodb: MongoDBConnection, redis: RedisConnection, discord_http: Arc<Client>) -> ResponseData {

    let context = InteractionContext::from_modal_submit_interaction(interaction, application.clone()).await?;
    let command = application.find_command(context.command_text.clone()).await.ok_or("Cannot find command".to_string())?;

    let guild_id = context.guild_id.ok_or("Cannot find guild_id".to_string())?;
    let config = mongodb.get_config(guild_id).await?;

    config.enabled.get(command.module.as_str()).ok_or("This module is disabled".to_string())?;

    (command.run)(context, mongodb, redis, discord_http).await

}