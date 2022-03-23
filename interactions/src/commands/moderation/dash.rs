use std::sync::Arc;
use twilight_http::Client;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::http::interaction::InteractionResponseType;
use database::models::config::GuildConfig;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use utils::check_type;
use utils::modals::{ModalBuilder, RepetitiveTextInput};
use crate::commands::ResponseData;
use crate::InteractionContext;

pub async fn run(interaction: InteractionContext, _: MongoDBConnection, _: RedisConnection, _: Arc<Client>, _: GuildConfig) -> ResponseData {

    let action = check_type!(
        interaction.options.get("action").ok_or("Unknown action".to_string())?,
        CommandOptionValue::String
    ).ok_or("Unknown action".to_string())?.clone();

    let modal = if action == "warn".to_string() {
        ModalBuilder::new(format!("a:warn-d"), "Warn".to_string())
            .add_repetitive_component(RepetitiveTextInput::Member)
            .add_repetitive_component(RepetitiveTextInput::Reason)
    } else if action == "mute".to_string() {
        ModalBuilder::new(format!("a:mute-d"), "Mute".to_string())
            .add_repetitive_component(RepetitiveTextInput::Member)
            .add_repetitive_component(RepetitiveTextInput::Duration)
            .add_repetitive_component(RepetitiveTextInput::Reason)
    } else if action == "kick".to_string() {
        ModalBuilder::new(format!("a:kick-d"), "Kick".to_string())
            .add_repetitive_component(RepetitiveTextInput::Member)
            .add_repetitive_component(RepetitiveTextInput::Reason)
    } else if action == "ban".to_string() {
        ModalBuilder::new(format!("a:ban-d"), "Ban".to_string())
            .add_repetitive_component(RepetitiveTextInput::Member)
            .add_repetitive_component(RepetitiveTextInput::Reason)
    } else { return Err("Unknown action".to_string()) };

    Ok((
        modal.to_interaction_response_data(),
        Some(InteractionResponseType::Modal)
    ))

}