use std::sync::Arc;
use twilight_model::application::interaction::{Interaction, InteractionType};
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use twilight_http::Client;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use utils::errors::Error;
use crate::{Application, extract};
use crate::commands::context::{InteractionContext, InteractionHelpers};
use crate::commands::ResponseData;
use crate::commands::options::LoadOptions;

async fn handle_command(
    interaction: Interaction,
    application: Application,
    mongodb: MongoDBConnection,
    redis: RedisConnection,
    discord_http: Arc<Client>
) -> ResponseData {
    let token = interaction.token.to_owned();
    let id = interaction.application_id.cast();

    let context: InteractionContext = interaction.try_into()?;
    let context = context.load_options(&application).await?;

    let command = application.find_command(&context.command_text)
        .await.ok_or("Cannot find command")?;

    extract!(context.interaction, guild_id);

    let config = mongodb.get_config(guild_id).await.map_err(Error::from)?;
    config.enabled.get(command.module.as_str()).ok_or("This module is disabled")?;

    let execute_as_slower = application.is_slower(&command.name).await
        && context.interaction.target_id().is_none();

    if execute_as_slower {
        tokio::spawn(async move {
            let response = (command.run)(
                context, mongodb, redis, discord_http.to_owned(), config
            ).await;

            // into_ok_or_err is unstable :/
            let response_data = match response {
                Ok((response, _)) => response,
                Err(error) => error.to_interaction_data_response()
            };

            let bytes_data = serde_json::to_vec(&response_data)
                .expect("Interaction response serialization error");

            discord_http.execute_webhook(id, &token)
                .payload_json(bytes_data.as_slice()).exec().await.ok();
        });

        Ok((InteractionResponseData {
            allowed_mentions: None,
            attachments: None,
            choices: None,
            components: None,
            content: None,
            custom_id: None,
            embeds: None,
            flags: Some(MessageFlags::EPHEMERAL),
            title: None,
            tts: None
        }, Some(InteractionResponseType::DeferredChannelMessageWithSource)))

    } else {
        (command.run)(context, mongodb, redis, discord_http, config).await
    }
}

pub async fn handle_interaction(
    interaction: Interaction,
    application: Application,
    mongodb: MongoDBConnection,
    redis: RedisConnection,
    discord_http: Arc<Client>
) -> InteractionResponse {
    if interaction.kind == InteractionType::Ping {
        return InteractionResponse {
            kind: InteractionResponseType::Pong,
            data: None
        }
    }

    let response_type_default =
        if interaction.kind == InteractionType::MessageComponent {
            InteractionResponseType::UpdateMessage
        } else { InteractionResponseType::ChannelMessageWithSource };

    let response = handle_command(
        interaction, application, mongodb, redis, discord_http
    ).await;

    match response {
        Ok((response, response_type)) => {
            InteractionResponse {
                kind: response_type.unwrap_or(response_type_default),
                data: Some(response)
            }
        },
        Err(error) => error.to_interaction_response()
    }
}
