use std::sync::Arc;
use twilight_model::application::interaction::{Interaction, InteractionType};
use twilight_http::Client;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use crate::context::Context;
use crate::commands::context::{InteractionContext, InteractionHelpers};
use crate::commands::ResponseData;
use crate::commands::options::LoadOptions;
use crate::extract;
use crate::utils::errors::Error;

async fn handle_command(
    interaction: Interaction,
    discord_http: Arc<Client>,
    context: Arc<Context>
) -> ResponseData {
    let token = interaction.token.to_owned();
    let id = interaction.application_id.cast();

    let interaction_ctx: InteractionContext = interaction.try_into()?;
    let interaction_ctx = interaction_ctx.load_options(&context.application).await?;

    let command = context.application.find_command(&interaction_ctx.command_text)
        .await.ok_or("Cannot find command")?;

    extract!(interaction_ctx.orginal, guild_id);

    let config = context.mongodb.get_config(guild_id).await.map_err(Error::from)?;
    if command.module != "settings" {
        config.enabled.get(command.module.as_str()).ok_or("This module is disabled")?;
    }

    let execute_as_slower = interaction_ctx.orginal.target_id().is_none()
        && context.application.is_slower(&command.name).await;

    if execute_as_slower {
        tokio::spawn(async move {
            let response = (command.run)(
                    interaction_ctx, context, discord_http.to_owned(), config
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
        (command.run)(interaction_ctx, context, discord_http, config).await
    }
}

pub async fn handle_interaction(
    interaction: Interaction,
    discord_http: Arc<Client>,
    context: Arc<Context>
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

        let response = handle_command(interaction, discord_http, context).await;

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
