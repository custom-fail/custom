use std::sync::Arc;
use mongodb::bson::doc;
use twilight_http::Client;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::http::interaction::InteractionResponseData;
use twilight_model::channel::message::MessageFlags;
use database::models::config::GuildConfig;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use utils::check_type;
use utils::errors::Error;
use crate::commands::context::InteractionContext;
use crate::commands::ResponseData;

pub async fn run(interaction: InteractionContext, mongodb: MongoDBConnection, _: RedisConnection, discord_http: Arc<Client>, _: GuildConfig) -> ResponseData {

    let case_id = *check_type!(
        interaction.options.get("id").ok_or("There is no case id")?,
        CommandOptionValue::Integer
    ).ok_or("Case id type not match")?;

    let removed_case = mongodb.cases.find_one_and_update(
        doc! { "index": case_id, "removed": false }, doc! { "$set": {"removed": true } }, None
    ).await.map_err(Error::from)?.ok_or("Cannot find case with selected id")?;

    Ok((InteractionResponseData {
        allowed_mentions: None,
        attachments: None,
        choices: None,
        components: None,
        content: Some("**Removed case**".to_string()),
        custom_id: None,
        embeds: Some(vec![removed_case.to_embed(discord_http).await?]),
        flags: Some(MessageFlags::EPHEMERAL),
        title: None,
        tts: None
    }, None))

}