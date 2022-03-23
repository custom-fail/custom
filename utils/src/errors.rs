use redis::RedisError;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use crate::embeds::EmbedBuilder;

#[derive(Debug)]
pub enum Error {
    DeserializeBody(twilight_http::response::DeserializeBodyError),
    MessageValidation(twilight_validate::message::MessageValidationError),
    ValidationError(twilight_validate::request::ValidationError),
    DiscordAPI(twilight_http::error::Error),
    MongoDB(mongodb::error::Error),
    Redis(redis::RedisError),
    Message(String)
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Self::Message(message)
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self::Message(message.to_string())
    }
}

impl From<redis::RedisError> for Error {
    fn from(error: RedisError) -> Self {
        Self::Redis(error)
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(error: mongodb::error::Error) -> Self {
        Self::MongoDB(error)
    }
}

impl From<mongodb::bson::de::Error> for Error {
    fn from(error: mongodb::bson::de::Error) -> Self {
        Error::from(mongodb::error::Error::from(error))
    }
}

impl From<twilight_http::response::DeserializeBodyError> for Error {
    fn from(error: twilight_http::response::DeserializeBodyError) -> Self {
        Self::DeserializeBody(error)
    }
}

impl From<twilight_http::error::Error> for Error {
    fn from(error: twilight_http::Error) -> Self {
        Self::DiscordAPI(error)
    }
}

impl From<twilight_validate::request::ValidationError> for Error {
    fn from(error: twilight_validate::request::ValidationError) -> Self {
        Self::ValidationError(error)
    }
}

impl From<twilight_validate::message::MessageValidationError> for Error {
    fn from(error: twilight_validate::message::MessageValidationError) -> Self {
        Self::MessageValidation(error)
    }
}

enum ResponseType {
    Message,
    Embed
}

impl Error {
    pub fn to_interaction_data_response(&self) -> InteractionResponseData {

        let error = match self {
            Error::DeserializeBody(error) =>
                (vec![format!("{:?}", error)], ResponseType::Embed),
            Error::ValidationError(error) =>
                (vec![error.to_string(), format!("{:?}", error)], ResponseType::Embed),
            Error::MessageValidation(error) =>
                (vec![error.to_string(), format!("{:?}", error)], ResponseType::Embed),
            Error::DiscordAPI(error) =>
                (vec![error.to_string(), format!("{:?}", error)], ResponseType::Embed),
            Error::MongoDB(error) => (vec![format!("{:?}", error)], ResponseType::Embed),
            Error::Redis(error) => (vec![format!("{:?}", error)], ResponseType::Embed),
            Error::Message(message) => (vec![message.clone()], ResponseType::Message)
        };

        if let ResponseType::Embed = error.1 {
            let description = format!("```{}```", error.0.join("``` ```"));
            EmbedBuilder::new()
                .title("Internal Server Error".to_string())
                .description(description)
                .to_interaction_response_data(true)
        } else {
            InteractionResponseData {
                allowed_mentions: None,
                attachments: None,
                choices: None,
                components: None,
                content: error.0.first().cloned(),
                custom_id: None,
                embeds: None,
                flags: Some(MessageFlags::EPHEMERAL),
                title: None,
                tts: None
            }
        }

    }

    pub fn to_interaction_response(&self) -> InteractionResponse {
        InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(self.to_interaction_data_response())
        }
    }
}