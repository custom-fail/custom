use twilight_model::application::callback::CallbackData;
use twilight_model::application::interaction::{ApplicationCommand, Interaction};
use twilight_model::channel::message::MessageFlags;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use serde::{Serialize, Deserialize};
use crate::Application;

#[derive(Serialize, Deserialize, Debug)]
pub struct InteractionResponse {
    r#type: u8,
    data: Option<CallbackData>
}

pub async fn handle_interaction(interaction: Interaction, application: Application, mongodb: MongoDBConnection, redis: RedisConnection) -> InteractionResponse {
    match interaction {
        Interaction::Ping(_) => InteractionResponse {
            r#type: 1,
            data: None
        },
        Interaction::ApplicationCommand(interaction) => InteractionResponse {
            r#type: 4,
            data: Some(commands_handler(interaction, application, mongodb, redis).await)
        },
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

async fn commands_handler(_interaction: Box<ApplicationCommand>, _application: Application, _mongodb: MongoDBConnection, _redis: RedisConnection) -> CallbackData {
    CallbackData {
        allowed_mentions: None,
        components: None,
        content: Some("test???".to_string()),
        embeds: None,
        flags: None,
        tts: None
    }
}