use crate::commands::ResponseData;
use crate::commands::context::InteractionContext;
use crate::context::Context;
use crate::models::config::GuildConfig;
use crate::utils::errors::Error;
use crate::{get_option, get_required_option};
use mongodb::bson::doc;
use std::sync::Arc;
use twilight_http::Client;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::InteractionResponseData;

pub async fn run(
    interaction: InteractionContext,
    context: Arc<Context>,
    discord_http: Arc<Client>,
    config: GuildConfig,
) -> ResponseData {
    let case_index = *get_required_option!(
        interaction.options.get("number"),
        CommandOptionValue::Integer
    );

    let removed_case = context
        .mongodb
        .cases
        .find_one_and_update(
            doc! {
                "guild_id": config.guild_id.to_string(),
                "index": case_index,
                "removed": false
            },
            doc! { "$set": {"removed": true } },
        )
        .await
        .map_err(Error::from)?
        .ok_or("Cannot find case with selected id")?;

    Ok((
        InteractionResponseData {
            allowed_mentions: None,
            attachments: None,
            choices: None,
            components: None,
            content: Some("**Removed case**".to_string()),
            custom_id: None,
            embeds: Some(vec![removed_case.to_embed(discord_http).await?]),
            flags: Some(MessageFlags::EPHEMERAL),
            title: None,
            tts: None,
        },
        None,
    ))
}
