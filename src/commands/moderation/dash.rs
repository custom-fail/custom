use std::sync::Arc;
use twilight_http::Client;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::http::interaction::InteractionResponseType;
use crate::commands::ResponseData;
use crate::context::Context;
use crate::{get_required_option, get_option};
use crate::commands::context::InteractionContext;
use crate::models::config::GuildConfig;
use crate::utils::errors::Error;
use crate::utils::modals::{ModalBuilder, RepetitiveTextInput};

pub async fn run(
    interaction: InteractionContext,
    _: Arc<Context>,
    _: Arc<Client>,
    _: GuildConfig
) -> ResponseData {
    let action = get_required_option!(
        interaction.options.get("action"), CommandOptionValue::String
    );

    let modal = if *action == "warn" {
        ModalBuilder::new("a:warn-d".to_string(), "Warn".to_string())
            .add_repetitive_component(RepetitiveTextInput::Member)
            .add_repetitive_component(RepetitiveTextInput::Reason)
    } else if *action == "mute" {
        ModalBuilder::new("a:mute-d".to_string(), "Mute".to_string())
            .add_repetitive_component(RepetitiveTextInput::Member)
            .add_repetitive_component(RepetitiveTextInput::Duration(true))
            .add_repetitive_component(RepetitiveTextInput::Reason)
    } else if *action == "kick" {
        ModalBuilder::new("a:kick-d".to_string(), "Kick".to_string())
            .add_repetitive_component(RepetitiveTextInput::Member)
            .add_repetitive_component(RepetitiveTextInput::Reason)
    } else if *action == "ban" {
        ModalBuilder::new("a:ban-d".to_string(), "Ban".to_string())
            .add_repetitive_component(RepetitiveTextInput::Member)
            .add_repetitive_component(RepetitiveTextInput::Duration(false))
            .add_repetitive_component(RepetitiveTextInput::Reason)
    } else { return Err(Error::from("Unknown action")) };

    Ok((
        modal.to_interaction_response_data(),
        Some(InteractionResponseType::Modal)
    ))
}