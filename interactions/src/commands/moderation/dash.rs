use std::sync::Arc;
use twilight_http::Client;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::http::interaction::InteractionResponseType;
use database::models::config::GuildConfig;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use utils::check_type;
use utils::errors::Error;
use utils::modals::{ModalBuilder, RepetitiveTextInput};
use crate::commands::ResponseData;
use crate::InteractionContext;

pub async fn run(interaction: InteractionContext, _: MongoDBConnection, _: RedisConnection, _: Arc<Client>, _: GuildConfig) -> ResponseData {

    let action = check_type!(
        interaction.options.get("action").ok_or("Unknown action")?,
        CommandOptionValue::String
    ).ok_or("Unknown action")?;

    let modal = if *action == "warn" {
        ModalBuilder::new("a:warn-d".to_string(), "Warn".to_string())
            .add_repetitive_component(RepetitiveTextInput::Member)
            .add_repetitive_component(RepetitiveTextInput::Reason)
    } else if *action == "mute" {
        ModalBuilder::new("a:mute-d".to_string(), "Mute".to_string())
            .add_repetitive_component(RepetitiveTextInput::Member)
            .add_repetitive_component(RepetitiveTextInput::Duration)
            .add_repetitive_component(RepetitiveTextInput::Reason)
    } else if *action == "kick" {
        ModalBuilder::new("a:kick-d".to_string(), "Kick".to_string())
            .add_repetitive_component(RepetitiveTextInput::Member)
            .add_repetitive_component(RepetitiveTextInput::Reason)
    } else if *action == "ban" {
        ModalBuilder::new("a:ban-d".to_string(), "Ban".to_string())
            .add_repetitive_component(RepetitiveTextInput::Member)
            .add_repetitive_component(RepetitiveTextInput::Reason)
    } else { return Err(Error::from("Unknown action")) };

    Ok((
        modal.to_interaction_response_data(),
        Some(InteractionResponseType::Modal)
    ))

}