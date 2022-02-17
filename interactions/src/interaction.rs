use std::sync::Arc;
use twilight_model::application::callback::CallbackData;
use twilight_model::application::interaction::{ApplicationCommand, Interaction};
use twilight_model::channel::message::MessageFlags;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use serde::{Serialize, Deserialize};
use twilight_http::Client;
use crate::Application;
use crate::commands::context::CommandContext;
use crate::commands::parse_slash_command_to_text;

#[derive(Serialize, Deserialize, Debug)]
pub struct InteractionResponse {
    r#type: u8,
    data: Option<CallbackData>
}

pub async fn handle_interaction(interaction: Interaction, application: Application, mongodb: MongoDBConnection, redis: RedisConnection, discord_http: Arc<Client>) -> InteractionResponse {
    match interaction {
        Interaction::Ping(_) => InteractionResponse {
            r#type: 1,
            data: None
        },
        Interaction::ApplicationCommand(interaction) => {
            let response = commands_handler(interaction,application, mongodb, redis, discord_http).await;
            match response {
                Ok(response) => InteractionResponse {
                    r#type: 4,
                    data: Some(response)
                },
                Err(error) => InteractionResponse {
                    r#type: 4,
                    data: Some(CallbackData {
                        allowed_mentions: None,
                        components: None,
                        content: Some(error),
                        embeds: None,
                        flags: Some(MessageFlags::EPHEMERAL),
                        tts: None
                    })
                }
            }
        }
        _ => InteractionResponse {
            r#type: 4,
            data: Some(CallbackData {
                allowed_mentions: None,
                components: None,
                content: Some("Not supported interaction type".to_string()),
                embeds: None,
                flags: Some(MessageFlags::EPHEMERAL),
                tts: None
            })
        }
    }
}

async fn commands_handler(interaction: Box<ApplicationCommand>, application: Application, mongodb: MongoDBConnection, redis: RedisConnection, discord_http: Arc<Client>) -> Result<CallbackData, String> {

    let command_vec = parse_slash_command_to_text(interaction.data.clone());
    let command_text = command_vec.clone().join(" ");
    let command = application.find_command(command_text.clone()).await.ok_or("Cannot find command")?;

    let guild_id = interaction.guild_id.ok_or("Cannot find guild_id".to_string())?;
    let config = mongodb.get_config(guild_id).await.map_err(|_| "Cannot find guild config".to_string())?;

    let context = CommandContext::from_command_data(interaction.clone(), (command_vec.clone(), command_text.clone()));

    config.enabled.get(command.module.as_str()).ok_or("This module is disabled".to_string())?;

    (command.run)(context, mongodb, redis, discord_http).await

}