use twilight_model::application::callback::CallbackData;
use twilight_model::application::interaction::Interaction;
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

pub async fn handle_interaction(interaction: Interaction, _application: Application, _mongodb: MongoDBConnection, _redis: RedisConnection) -> InteractionResponse {
    match interaction {
        Interaction::Ping(_) => InteractionResponse {
            r#type: 1,
            data: None
        },
        Interaction::ApplicationCommand(_interaction) => {
            InteractionResponse {
                r#type: 4,
                data: Some(CallbackData {
                    allowed_mentions: None,
                    components: None,
                    content: Some("test???".to_string()),
                    embeds: None,
                    flags: Some(MessageFlags::EPHEMERAL),
                    tts: None
                })
            }
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