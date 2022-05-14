use redis::RedisError;
use twilight_model::channel::message::MessageFlags;
use twilight_model::datetime::TimestampParseError;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use crate::embeds::EmbedBuilder;

#[derive(Debug)]
pub enum Error {
    Debug(Vec<String>),
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
        Self::Debug(vec![format!("{:?}", error)])
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(error: mongodb::error::Error) -> Self {
        Self::Debug(vec![format!("{:?}", error)])
    }
}

impl From<mongodb::bson::de::Error> for Error {
    fn from(error: mongodb::bson::de::Error) -> Self {
        Self::Debug(vec![format!("{:?}", error)])
    }
}

impl From<twilight_http::response::DeserializeBodyError> for Error {
    fn from(error: twilight_http::response::DeserializeBodyError) -> Self {
        Self::Debug(vec![format!("{:?}", error)])
    }
}

impl From<twilight_http::error::Error> for Error {
    fn from(error: twilight_http::Error) -> Self {
        Self::Debug(vec![error.to_string(), format!("{:?}", error)])
    }
}

impl From<twilight_validate::request::ValidationError> for Error {
    fn from(error: twilight_validate::request::ValidationError) -> Self {
        Self::Debug(vec![error.to_string(), format!("{:?}", error)])
    }
}

impl From<twilight_validate::message::MessageValidationError> for Error {
    fn from(error: twilight_validate::message::MessageValidationError) -> Self {
        Self::Debug(vec![error.to_string(), format!("{:?}", error)])
    }
}

impl From<TimestampParseError> for Error {
    fn from(error: TimestampParseError) -> Self {
        Self::Debug(vec![error.to_string(), format!("{:?}", error)])
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Debug(vec![format!("{:?}", error)])
    }
}

impl Error {
    pub fn to_interaction_data_response(&self) -> InteractionResponseData {

        match self {
            Error::Debug(errors) => {
                let description = format!("```{}```", errors.join("``` ```"));
                EmbedBuilder::new()
                    .title("Internal Server Error".to_string())
                    .description(description)
                    .to_interaction_response_data(true)
            },
            Error::Message(message) => {
                InteractionResponseData {
                    allowed_mentions: None,
                    attachments: None,
                    choices: None,
                    components: None,
                    content: Some(message.to_owned()),
                    custom_id: None,
                    embeds: None,
                    flags: Some(MessageFlags::EPHEMERAL),
                    title: None,
                    tts: None
                }
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