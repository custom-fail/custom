use std::sync::Arc;
use twilight_model::application::interaction::{ApplicationCommand, Interaction, InteractionType, MessageComponentInteraction};
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use twilight_http::Client;
use twilight_model::application::interaction::modal::ModalSubmitInteraction;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use utils::errors::Error;
use crate::{Application, Command};
use crate::commands::context::InteractionContext;
use crate::commands::{parse_slash_command_to_text, ResponseData};

pub async fn handle_command(
    interaction: Interaction,
    application: Application,
    mongodb: MongoDBConnection,
    redis: RedisConnection,
    discord_http: Arc<Client>
) -> ResponseData {

    let token = interaction.token().to_owned();
    let id = interaction.application_id().cast().to_owned();

    let (context, command) = match interaction {
        Interaction::ApplicationCommand(interaction) => {
            commands_handler(interaction, &application).await?
        }
        Interaction::MessageComponent(interaction) => {
            component_handler(interaction, &application).await?
        },
        Interaction::ModalSubmit(interaction) => {
            modal_handler(interaction, &application).await?
        }
        _ => return Err(Error::from("Not supported interaction type"))
    };

    let guild_id = context.guild_id.ok_or("Cannot find guild_id")?;
    let config = mongodb.get_config(guild_id).await.map_err(Error::from)?;

    config.enabled.get(command.module.as_str()).ok_or("This module is disabled")?;

    if application.is_slower(&command.name).await && context.target_id.is_none() {
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

            discord_http.execute_webhook(id, &*token)
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
    match interaction {
        Interaction::Ping(_) => InteractionResponse {
            kind: InteractionResponseType::Pong,
            data: None
        },
        _ => {
            let response_type_default =
                if interaction.kind() == InteractionType::MessageComponent {
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
    }
}

type ExecutionContextResult = Result<(InteractionContext, Command), Error>;

async fn component_handler(
    interaction: Box<MessageComponentInteraction>,
    application: &Application
) -> ExecutionContextResult {

    let context = InteractionContext::from_message_component_interaction(
        interaction, application
    ).await?;
    let command = application.find_command(&context.command_text)
        .await.ok_or("Cannot find command")?;

    Ok((context, command))

}

async fn commands_handler(
    interaction: Box<ApplicationCommand>,
    application: &Application
) -> ExecutionContextResult {

    let command_vec = parse_slash_command_to_text(interaction.data.clone());
    let command_text = command_vec.clone().join(" ");
    let command = application.find_command(&command_text)
        .await.ok_or("Cannot find command")?;

    let context = InteractionContext::from_command_data(
        interaction, (command_vec.clone(), command_text.clone())
    )?;

    Ok((context, command))

}

async fn modal_handler(
    interaction: Box<ModalSubmitInteraction>,
    application: &Application
) -> ExecutionContextResult {

    let context = InteractionContext::from_modal_submit_interaction(
        interaction, application
    ).await?;
    let command = application.find_command(&context.command_text)
        .await.ok_or("Cannot find command")?;

    Ok((context, command))

}